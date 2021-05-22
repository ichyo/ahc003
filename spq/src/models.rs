use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::io::{BufRead, Write};
use std::ops::{Index, IndexMut};

pub const NUM_TURN: usize = 1000;
pub const GRID_LEN: usize = 30;

pub trait Environment {
    fn next_query(&self) -> Option<Query>;
    fn do_answer(&mut self, path: Vec<Dir>) -> f64;
}

#[derive(Debug, Clone, Copy)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn iter() -> impl Iterator<Item = Dir> {
        [Dir::Up, Dir::Left, Dir::Down, Dir::Right].iter().cloned()
    }
}

pub struct Grid<T: Copy>([[T; GRID_LEN]; GRID_LEN]);

impl<T: Copy> Grid<T> {
    pub fn new(value: T) -> Grid<T> {
        Grid([[value; GRID_LEN]; GRID_LEN])
    }
}

impl<T: Copy> Index<Pos> for Grid<T> {
    type Output = T;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.0[index.r as usize][index.c as usize]
    }
}

impl<T: Copy> IndexMut<Pos> for Grid<T> {
    fn index_mut(&mut self, index: Pos) -> &mut Self::Output {
        &mut self.0[index.r as usize][index.c as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos {
    pub r: u8,
    pub c: u8,
}

impl Pos {
    pub fn new(r: u8, c: u8) -> Pos {
        Pos { r, c }
    }

    pub fn move_to(&self, d: Dir) -> Option<Pos> {
        match d {
            Dir::Up => {
                if self.r == 0 {
                    None
                } else {
                    Some(Pos::new(self.r - 1, self.c))
                }
            }
            Dir::Down => {
                if self.r as usize == GRID_LEN - 1 {
                    None
                } else {
                    Some(Pos::new(self.r + 1, self.c))
                }
            }
            Dir::Left => {
                if self.c == 0 {
                    None
                } else {
                    Some(Pos::new(self.r, self.c - 1))
                }
            }
            Dir::Right => {
                if self.c as usize == GRID_LEN - 1 {
                    None
                } else {
                    Some(Pos::new(self.r, self.c + 1))
                }
            }
        }
    }
}

struct GridGraph {
    h_cost: [[u32; GRID_LEN - 1]; GRID_LEN],
    v_cost: [[u32; GRID_LEN]; GRID_LEN - 1],
}

impl GridGraph {
    fn get_cost(&self, p: Pos, d: Dir) -> u32 {
        assert!(p.move_to(d).is_some());
        match d {
            Dir::Up => self.v_cost[p.r as usize - 1][p.c as usize],
            Dir::Down => self.v_cost[p.r as usize][p.c as usize],
            Dir::Left => self.h_cost[p.r as usize][p.c as usize - 1],
            Dir::Right => self.h_cost[p.r as usize][p.c as usize],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Query {
    src: Pos,
    dest: Pos,
}

#[derive(Debug, Clone)]
struct QueryParam {
    query: Query,
    res_factor: f64,
}

pub struct Simulator {
    turn: usize,
    graph: GridGraph,
    queries: Vec<QueryParam>,
    visited: Grid<usize>,
    score: f64,
}

impl Simulator {
    fn compute_shortest_path(&self, query: &Query) -> u32 {
        let src = query.src;
        let dest = query.dest;
        let mut dist: Grid<u32> = Grid::new(0);
        let mut prev: Grid<Dir> = Grid::new(Dir::Up);
        let mut queue = BinaryHeap::new();
        dist[src] = 0;
        queue.push(Reverse((0, src)));
        while let Some(Reverse((d, p))) = queue.pop() {
            if p == dest {
                break;
            }
            if dist[p] != d {
                continue;
            }
            for dir in Dir::iter() {
                if let Some(q) = p.move_to(dir) {
                    if dist[q] > d + self.graph.get_cost(p, dir) {
                        dist[q] = d + self.graph.get_cost(p, dir);
                        prev[q] = dir;
                        queue.push(Reverse((dist[q], q)));
                    }
                }
            }
        }
        dist[dest]
    }

    fn compute_path_length(&mut self, path: &[Dir]) -> Result<u32, String> {
        let src = self.queries[self.turn].query.src;
        let dest = self.queries[self.turn].query.dest;
        let mut p = src;
        let mut sum = 0;
        for &d in path {
            if self.visited[p] == self.turn {
                return Err(format!(
                    "visiting ({},{}) twice (query {})",
                    p.r,
                    p.c,
                    self.turn + 1
                ));
            }
            self.visited[p] = self.turn;

            let np = match p.move_to(d) {
                Some(np) => np,
                None => return Err(format!("going outside the map (query {})", self.turn + 1)),
            };

            sum += self.graph.get_cost(np, d);
            p = np;
        }
        if p != dest {
            return Err(format!("not an s-t path (query {})", self.turn + 1));
        }
        Ok(sum)
    }
}

impl Environment for Simulator {
    fn next_query(&self) -> Option<Query> {
        if self.turn < NUM_TURN {
            Some(self.queries[self.turn].query.clone())
        } else {
            None
        }
    }

    fn do_answer(&mut self, path: Vec<Dir>) -> f64 {
        let query = self.queries[self.turn].clone();
        let length = self.compute_path_length(&path).expect("invalid path");
        let best = self.compute_shortest_path(&query.query);
        let ratio = length as f64 / best as f64;
        assert!(length <= best);
        self.score = self.score * 0.998 + ratio;
        self.turn += 1;
        ratio * query.res_factor
    }
}

pub struct RemoteEnvironment<R: BufRead, W: Write> {
    turn: usize,
    reader: R,
    writer: W,
    next_query: Option<Query>,
}

impl<R: BufRead, W: Write> RemoteEnvironment<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        let mut e = RemoteEnvironment {
            turn: 0,
            next_query: None,
            reader,
            writer,
        };
        e.read_next_query();
        e
    }

    fn read_next_query(&mut self) {
        let mut next_query = String::new();
        self.reader
            .read_line(&mut next_query)
            .expect("read query failed");
        let v: Vec<u8> = next_query
            .split_whitespace()
            .map(|x| {
                x.parse::<u8>()
                    .map_err(|e| format!("invalid query {}: {}", next_query, e))
                    .unwrap()
            })
            .collect();
        if v.len() != 4 {
            panic!("invalid query {}", next_query);
        }
        self.next_query = Some(Query {
            src: Pos::new(v[0], v[1]),
            dest: Pos::new(v[2], v[3]),
        })
    }
}

impl<R: BufRead, W: Write> Environment for RemoteEnvironment<R, W> {
    fn next_query(&self) -> Option<Query> {
        self.next_query.clone()
    }

    fn do_answer(&mut self, path: Vec<Dir>) -> f64 {
        writeln!(
            self.writer,
            "{}",
            path.iter()
                .map(|d| match d {
                    Dir::Up => 'U',
                    Dir::Down => 'D',
                    Dir::Left => 'L',
                    Dir::Right => 'R',
                })
                .collect::<String>()
        )
        .expect("write failed");
        self.writer.flush().expect("flush failed");

        let mut score = String::new();
        self.reader
            .read_line(&mut score)
            .expect("read score failed");
        let score = score
            .parse::<f64>()
            .map_err(|e| format!("invalid score {}: {}", score, e))
            .unwrap();

        self.turn += 1;

        if self.turn < NUM_TURN {
            self.read_next_query();
        } else {
            self.next_query = None;
        }

        score
    }
}
