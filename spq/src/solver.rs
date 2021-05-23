use crate::algorithms::compute_shortest_path;
use crate::models::*;
use rand::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::convert::TryInto;
use std::ops::Index;
use std::time::{Duration, Instant};

struct Record {
    query: Query,
    path: Vec<Dir>,
    response: u32,
    visited: FxHashSet<EdgeIndex>,
}

impl Record {
    fn new(query: &Query, path: &[Dir], response: u32) -> Self {
        let mut visited = FxHashSet::default();
        let mut cur = query.src;
        for &dir in path {
            let edge = EdgeIndex::from_move(cur, dir);
            visited.insert(edge);
            cur = cur.move_to(dir).unwrap();
        }
        assert!(cur == query.dest);
        Record {
            query: query.clone(),
            path: path.iter().cloned().collect(),
            response,
            visited,
        }
    }
}

struct GraphEstimator2 {
    costs: GridLines<[u32; 2]>,
    mid_x: GridLines<u8>,
    records: Vec<Record>,
    // Cache for estimation
    visit_counts: Vec<GridLines<[u32; 2]>>,
    total_costs: Vec<u32>,
    visited_turns: FxHashMap<LineIndex, FxHashSet<u16>>,
    loss: i64,
}

impl Index<EdgeIndex> for GraphEstimator2 {
    type Output = u32;

    fn index(&self, index: EdgeIndex) -> &Self::Output {
        let mid_x = self.mid_x[index.line];
        if index.x < mid_x {
            &self.costs[index.line][0]
        } else {
            &self.costs[index.line][1]
        }
    }
}

impl GraphEstimator2 {
    fn new() -> GraphEstimator2 {
        GraphEstimator2 {
            costs: GridLines::new([5000, 5000]),
            mid_x: GridLines::new(GRID_LEN as u8 / 2),
            records: Vec::new(),
            visit_counts: Vec::new(),
            total_costs: Vec::new(),
            visited_turns: FxHashMap::default(),
            loss: 0,
        }
    }

    fn validate_cache(&self) {
        assert!(self.records.len() == self.visit_counts.len());
        let turn = self.records.len();
        let mut actual_loss = 0i64;
        for i in 0..turn {
            let mut cost_sum = 0;
            for &edge in &self.records[i].visited {
                let cost = self.index(edge);
                cost_sum += cost;
            }
            assert!(
                self.total_costs[i] == cost_sum,
                "i={} total_costs={} cost_sum={}",
                i,
                self.total_costs[i],
                cost_sum
            );
            actual_loss += (cost_sum as i64 - self.records[i].response as i64).abs();
        }
        assert!(actual_loss == self.loss);
    }

    fn insert_new_record(&mut self, query: &Query, path: &[Dir], response: u32) {
        self.records.push(Record::new(query, path, response));
        self.update_estimation();
    }

    fn update_estimation(&mut self) {
        let start = Instant::now();
        let time_limit = Duration::from_micros(1500); // TODO: more dynamic

        assert!(
            self.visit_counts.len() + 1 == self.records.len(),
            "{} + 1 != {}",
            self.visit_counts.len(),
            self.records.len()
        );
        let this_turn = self.records.len() - 1;
        let mut visit_count = GridLines::new([0; 2]);
        let mut total_cost = 0u32;

        for &edge in &self.records[this_turn].visited {
            let cost = self.index(edge);
            total_cost += cost;

            self.visited_turns
                .entry(edge.line)
                .or_default()
                .insert(this_turn as u16);

            if edge.x < self.mid_x[edge.line] {
                visit_count[edge.line][0] += 1;
            } else {
                visit_count[edge.line][1] += 1;
            }
        }

        self.loss += (total_cost as i64 - self.records[this_turn].response as i64).abs();
        self.visit_counts.push(visit_count);
        self.total_costs.push(total_cost);

        //self.validate_cache(); // TODO: remove when submit

        let mut rng = thread_rng();
        let mut loops = 0;
        let mut updates = 0;
        let start_loss = self.loss;
        while start.elapsed() <= time_limit {
            loops += 1;
            let update_type = rng.gen_range(0, 2);
            if update_type == 0 {
                let line = LineIndex::choose(&mut rng);
                let part = rng.gen_range(0, 2);
                let step = 100;
                let sign: i64 = if rng.gen::<bool>() { 1 } else { -1 };
                let cur_cost = self.costs[line][part];
                let next_cost = cur_cost as i64 + sign * step;
                if next_cost < 0 || next_cost > 9000 {
                    continue;
                }
                let mut loss_diff = 0i64;
                if let Some(turns) = self.visited_turns.get(&line) {
                    for &turn in turns {
                        let turn = turn as usize;
                        let visit_count = &self.visit_counts[turn];
                        let response = self.records[turn].response as i64;
                        let cur_total_cost = self.total_costs[turn] as i64;
                        let new_total_cost = self.total_costs[turn] as i64
                            + sign * step * visit_count[line][part] as i64;
                        loss_diff -= (cur_total_cost - response).abs();
                        loss_diff += (new_total_cost - response).abs();
                    }
                }
                if loss_diff < 0 {
                    self.costs[line][part] = next_cost as u32;
                    self.loss += loss_diff;
                    if let Some(turns) = self.visited_turns.get(&line) {
                        for &turn in turns {
                            let visit_count = &self.visit_counts[turn as usize];
                            let new_total_cost = self.total_costs[turn as usize] as i64
                                + sign * step * visit_count[line][part] as i64;
                            self.total_costs[turn as usize] = new_total_cost as u32;
                        }
                    }
                    updates += 1;
                }
            } else {
                let line = LineIndex::choose(&mut rng);
                let sign: i8 = if rng.gen::<bool>() { 1 } else { -1 };
                let cur_mid_x = self.mid_x[line];
                let next_mid_x = self.mid_x[line] as i8 + sign;
                if next_mid_x <= 0 || next_mid_x >= GRID_LEN as i8 - 1 {
                    continue;
                }
                let next_mid_x = next_mid_x as u8;

                // sign == +1 -> cur_mid_x moves from part 1 to part 0
                // sign == -1 => new_mid_x moves from part 0 to part 1
                let edge = if sign == 1 {
                    EdgeIndex::new(line, cur_mid_x)
                } else {
                    EdgeIndex::new(line, next_mid_x)
                };
                let (old_part, new_part) = if sign == 1 { (1, 0) } else { (0, 1) };

                let mut loss_diff = 0i64;
                if let Some(turns) = self.visited_turns.get(&line) {
                    for &turn in turns {
                        let turn = turn as usize;
                        if !self.records[turn].visited.contains(&edge) {
                            continue;
                        }

                        let response = self.records[turn as usize].response as i64;
                        let cur_total_cost = self.total_costs[turn as usize] as i64;
                        let cost_diff =
                            self.costs[line][new_part] as i64 - self.costs[line][old_part] as i64;

                        loss_diff -= (cur_total_cost - response).abs();
                        loss_diff += (cur_total_cost + cost_diff - response).abs();
                    }
                }
                if loss_diff < 0 {
                    if let Some(turns) = self.visited_turns.get(&line) {
                        for &turn in turns {
                            let turn = turn as usize;
                            if !self.records[turn].visited.contains(&edge) {
                                continue;
                            }

                            let cur_total_cost = self.total_costs[turn as usize] as i64;
                            let cost_diff = self.costs[line][new_part] as i64
                                - self.costs[line][old_part] as i64;
                            let new_total_cost: u32 =
                                (cur_total_cost + cost_diff).try_into().unwrap();

                            assert!(
                                self.visit_counts[turn][line][old_part] > 0,
                                "{:?} {}",
                                self.visit_counts[turn][line],
                                self.mid_x[line],
                            );
                            self.visit_counts[turn][line][old_part] -= 1;
                            self.visit_counts[turn][line][new_part] += 1;

                            self.total_costs[turn] = new_total_cost;
                        }
                    }
                    self.mid_x[line] = next_mid_x;
                    self.loss += loss_diff;
                    updates += 1;
                }
            }
        }

        // self.validate_cache(); // TODO: remove when submit

        trace!(
            "Finish updating estimation. loss={:6}->{:6} loops={:6} updates={:6}",
            start_loss,
            self.loss,
            loops,
            updates
        );
    }
}

struct GraphEstimator {
    estimation: GridLines<u32>,
    records: Vec<Record>,
}

impl Index<EdgeIndex> for GraphEstimator {
    type Output = u32;

    fn index(&self, index: EdgeIndex) -> &Self::Output {
        &self.estimation[index.line]
    }
}

impl GraphEstimator {
    fn new() -> GraphEstimator {
        GraphEstimator {
            estimation: GridLines::new(5000),
            records: Vec::new(),
        }
    }

    fn insert_new_record(&mut self, query: &Query, path: &[Dir], response: u32) {
        self.records.push(Record::new(query, path, response));
        self.update_estimation();
    }

    fn update_estimation(&mut self) {
        let n = self.records.len();
        if n == 0 {
            return;
        }
        let mut count = vec![GridLines::new(0); n]; // row
        let mut length = vec![0i64; n];
        for i in 0..n {
            let mut p = self.records[i].query.src;
            for &d in &self.records[i].path {
                let edge = EdgeIndex::from_move(p, d);
                length[i] += self.estimation[edge.line] as i64;
                count[i][edge.line] += 1;
                p = p.move_to(d).unwrap();
            }
            assert!(p == self.records[i].query.dest);
        }
        let mut diff = 0i64;
        for i in 0..n {
            diff += (length[i] as i64 - self.records[i].response as i64).abs();
        }

        let mut updated = true;
        let mut loops = 0;
        trace!("Start updating estimation from diff={}", diff);
        while updated {
            loops += 1;
            updated = false;
            for sign in &[-100i64, 100i64] {
                for line in LineIndex::iter() {
                    let est = self.estimation[line];
                    let new_est = est as i64 + sign;
                    if new_est < 0 || new_est > 9000 {
                        continue;
                    }

                    let mut new_diff = 0;
                    for i in 0..n {
                        let count = count[i][line];
                        new_diff += (length[i] as i64 + sign * count
                            - self.records[i].response as i64)
                            .abs();
                    }
                    if new_diff < diff {
                        for i in 0..n {
                            let count = count[i][line];
                            length[i] += sign * count;
                        }
                        self.estimation[line] = new_est as u32;
                        diff = new_diff;
                        updated = true;
                    }
                }
            }
        }
        trace!("Finish updating estimation. diff={} loops={}", diff, loops);
    }
}

pub fn run_solver<E: Environment>(env: &mut E) {
    let mut estimator = GraphEstimator2::new();
    while let Some(query) = env.next_query() {
        trace!(
            "Start processing a query ({:2}, {:2}) -> ({:2}, {:2}) width={:2} height={:2}",
            query.src.r,
            query.src.c,
            query.dest.r,
            query.dest.c,
            query.width(),
            query.height()
        );
        let (path, estimated_length) = compute_shortest_path(&estimator, query.src, query.dest);
        trace!(
            "Sending a path: {}",
            path.iter().map(|d| d.to_char()).collect::<String>()
        );
        let response = env.do_answer(&path);
        trace!(
            "Got a response. length={:6} esimated={:6} ratio={:.2}",
            response,
            estimated_length,
            response as f64 / estimated_length as f64
        );
        estimator.insert_new_record(&query, &path, response);
    }
}
