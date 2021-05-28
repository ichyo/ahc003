use crate::algorithms::compute_shortest_path;
use crate::algorithms::Graph;
use crate::models::*;
use rand::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::convert::TryInto;
use std::ops::Index;
use std::time::{Duration, Instant};

const NORM_P: u32 = 2;

const LINE_COST_LB: i64 = 1000;
const LINE_COST_UB: i64 = 9000;
const EDGE_COST_LB: i64 = -300;
const EDGE_COST_UB: i64 = 300;

const STEP: i64 = 100;
const START_TEMP: f64 = 100000.0;
const END_TEMP: f64 = 100.0;

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

struct GraphEstimator {
    line_costs: GridLines<[u32; 2]>,
    edge_costs: GridGraph<i32>,
    mid_x: GridLines<u8>,
    records: Vec<Record>,
    // Cache for estimation
    visit_counts: Vec<GridLines<[u32; 2]>>,
    total_costs: Vec<u32>,
    visited_turns_per_line: FxHashMap<LineIndex, FxHashSet<u16>>,
    visited_turns_per_edge: FxHashMap<EdgeIndex, FxHashSet<u16>>,
    loss: i64,
    time_limit: Duration,
}

impl Graph<u32> for GraphEstimator {
    fn get_cost(&self, edge: EdgeIndex) -> u32 {
        let mid_x = self.mid_x[edge.line];
        let line_cost = if edge.x < mid_x {
            self.line_costs[edge.line][0]
        } else {
            self.line_costs[edge.line][1]
        } as i32;
        let edge_cost = self.edge_costs[edge];
        let result = (line_cost + edge_cost).try_into().unwrap();
        result
    }
}

impl GraphEstimator {
    fn new(time_limit: Duration) -> GraphEstimator {
        GraphEstimator {
            line_costs: GridLines::new([1000, 1000]),
            edge_costs: GridGraph::new(0),
            mid_x: GridLines::new(GRID_LEN as u8 / 2),
            records: Vec::new(),
            visit_counts: Vec::new(),
            total_costs: Vec::new(),
            visited_turns_per_line: FxHashMap::default(),
            visited_turns_per_edge: FxHashMap::default(),
            loss: 0,
            time_limit,
        }
    }

    fn validate_cache(&self) {
        assert!(self.records.len() == self.visit_counts.len());
        let turn = self.records.len();
        let mut actual_loss = 0i64;
        for i in 0..turn {
            let mut cost_sum = 0;
            for &edge in &self.records[i].visited {
                let cost = self.get_cost(edge);
                cost_sum += cost;
            }
            assert!(
                self.total_costs[i] == cost_sum,
                "i={} total_costs={} cost_sum={}",
                i,
                self.total_costs[i],
                cost_sum
            );
            actual_loss += (cost_sum as i64 - self.records[i].response as i64)
                .abs()
                .pow(2);
        }
        assert!(actual_loss == self.loss);
    }

    fn insert_new_record(&mut self, query: &Query, path: &[Dir], response: u32) {
        let this_turn = self.records.len();
        self.records.push(Record::new(query, path, response));

        let mut visit_count = GridLines::new([0; 2]);
        let mut total_cost = 0u32;

        for &edge in &self.records[this_turn].visited {
            let cost = self.get_cost(edge);
            total_cost += cost;

            self.visited_turns_per_line
                .entry(edge.line)
                .or_default()
                .insert(this_turn as u16);

            self.visited_turns_per_edge
                .entry(edge)
                .or_default()
                .insert(this_turn as u16);

            if edge.x < self.mid_x[edge.line] {
                visit_count[edge.line][0] += 1;
            } else {
                visit_count[edge.line][1] += 1;
            }
        }

        self.loss += (total_cost as i64 - self.records[this_turn].response as i64)
            .abs()
            .pow(NORM_P);
        self.visit_counts.push(visit_count);
        self.total_costs.push(total_cost);

        self.update_estimation();
    }

    fn update_estimation(&mut self) {
        let start = Instant::now();
        let time_limit = self.time_limit * 90 / 100 / 1000;

        let mut loops = 0;
        let mut updates_type0 = 0;
        let mut updates_type1 = 0;
        let mut updates_type2 = 0;
        let start_loss = self.loss;

        loop {
            let elapsed = start.elapsed();

            let ratio = elapsed.as_secs_f64() / time_limit.as_secs_f64();

            if ratio >= 1.0 {
                break;
            }

            let temp = START_TEMP + (END_TEMP - START_TEMP) * ratio;

            loops += 1;
            let mut rng = thread_rng();
            let update_type = rng.gen_range(0, 3);
            if update_type == 0 {
                let line = LineIndex::choose(&mut rng);
                let part = rng.gen_range(0, 2);
                let sign: i64 = if rng.gen::<bool>() { 1 } else { -1 };
                let cur_cost = self.line_costs[line][part];
                let next_cost = cur_cost as i64 + sign * STEP;
                if next_cost < LINE_COST_LB || next_cost > LINE_COST_UB {
                    continue;
                }

                let mut loss_diff = 0i64;
                let mut loss_diff_updated = false;

                if let Some(turns) = self.visited_turns_per_line.get(&line) {
                    for &turn in turns {
                        let turn = turn as usize;
                        let visit_counts = &self.visit_counts[turn];
                        let visit_count = visit_counts[line][part];

                        if visit_count == 0 {
                            continue;
                        }

                        let response = self.records[turn].response as i64;
                        let cur_total_cost = self.total_costs[turn] as i64;
                        let new_total_cost =
                            self.total_costs[turn] as i64 + sign * STEP * visit_count as i64;
                        loss_diff -= (cur_total_cost - response).abs().pow(NORM_P);
                        loss_diff += (new_total_cost - response).abs().pow(NORM_P);
                        loss_diff_updated = true;
                    }
                }

                if !loss_diff_updated {
                    continue;
                }

                let prob = (-loss_diff as f64 / temp).exp();
                if rng.gen::<f64>() < prob {
                    self.line_costs[line][part] = next_cost as u32;
                    self.loss += loss_diff;
                    if let Some(turns) = self.visited_turns_per_line.get(&line) {
                        for &turn in turns {
                            let visit_count = &self.visit_counts[turn as usize];
                            let new_total_cost = self.total_costs[turn as usize] as i64
                                + sign * STEP * visit_count[line][part] as i64;
                            self.total_costs[turn as usize] = new_total_cost as u32;
                        }
                    }
                    updates_type0 += 1;
                }
            } else if update_type == 1 {
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
                if let Some(turns) = self.visited_turns_per_line.get(&line) {
                    for &turn in turns {
                        let turn = turn as usize;
                        if !self.records[turn].visited.contains(&edge) {
                            continue;
                        }

                        let response = self.records[turn as usize].response as i64;
                        let cur_total_cost = self.total_costs[turn as usize] as i64;
                        let cost_diff = self.line_costs[line][new_part] as i64
                            - self.line_costs[line][old_part] as i64;

                        loss_diff -= (cur_total_cost - response).abs().pow(NORM_P);
                        loss_diff += (cur_total_cost + cost_diff - response).abs().pow(NORM_P);
                    }
                }
                let prob = (-loss_diff as f64 / temp).exp();
                if rng.gen::<f64>() < prob {
                    if let Some(turns) = self.visited_turns_per_line.get(&line) {
                        for &turn in turns {
                            let turn = turn as usize;
                            if !self.records[turn].visited.contains(&edge) {
                                continue;
                            }

                            let cur_total_cost = self.total_costs[turn as usize] as i64;
                            let cost_diff = self.line_costs[line][new_part] as i64
                                - self.line_costs[line][old_part] as i64;
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
                    updates_type1 += 1;
                }
            } else {
                let edge = EdgeIndex::choose(&mut rng);
                let sign: i64 = if rng.gen::<bool>() { 1 } else { -1 };
                let cur_cost = self.edge_costs[edge];
                let next_cost = cur_cost as i64 + sign * STEP;
                if next_cost < EDGE_COST_LB || next_cost > EDGE_COST_UB {
                    continue;
                }

                let mut loss_diff = 0i64;
                let mut loss_diff_updated = false;

                if let Some(turns) = self.visited_turns_per_edge.get(&edge) {
                    for &turn in turns {
                        let turn = turn as usize;

                        let response = self.records[turn].response as i64;
                        let cur_total_cost = self.total_costs[turn] as i64;
                        let new_total_cost = self.total_costs[turn] as i64 + sign * STEP as i64;
                        loss_diff -= (cur_total_cost - response).abs().pow(NORM_P);
                        loss_diff += (new_total_cost - response).abs().pow(NORM_P);
                        loss_diff_updated = true;
                    }
                }

                if !loss_diff_updated {
                    continue;
                }

                let prob = (-loss_diff as f64 / temp).exp();
                if rng.gen::<f64>() < prob {
                    self.edge_costs[edge] = next_cost as i32;
                    self.loss += loss_diff;
                    if let Some(turns) = self.visited_turns_per_edge.get(&edge) {
                        for &turn in turns {
                            let new_total_cost =
                                self.total_costs[turn as usize] as i64 + sign * STEP as i64;
                            self.total_costs[turn as usize] = new_total_cost as u32;
                        }
                    }
                    updates_type2 += 1;
                }
            }
        }

        trace!(
            "Finish updating estimation. loss={:6}->{:6}({:6}) loops={:4} updates=({:3}, {:3}, {:3})",
            start_loss,
            self.loss,
            self.loss - start_loss,
            loops,
            updates_type0,
            updates_type1,
            updates_type2
        );
        trace!("costs={:?} mid_x={:?}", self.line_costs, self.mid_x);
    }
}

pub fn run_solver<E: Environment>(env: &mut E, time_limit: Duration) {
    let mut estimator = GraphEstimator::new(time_limit);
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
        estimator.validate_cache();
    }
    debug!("line_costs={:?}", estimator.line_costs);
    debug!("edge_costs={:?}", estimator.edge_costs);
    debug!("mid_x={:?}", estimator.mid_x);
}
