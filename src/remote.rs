use crate::models::*;
use std::io::{BufRead, Write};
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
