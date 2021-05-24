use rand::prelude::*;
use std::cmp::Ordering;
use std::ops::{Index, IndexMut};

pub const NUM_TURN: usize = 1000;
pub const GRID_LEN: usize = 30;

pub trait Environment {
    fn next_query(&self) -> Option<Query>;
    fn do_answer(&mut self, path: &[Dir]) -> u32;
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
    pub fn rev(&self) -> Dir {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
    pub fn to_char(&self) -> char {
        match self {
            Dir::Up => 'U',
            Dir::Down => 'D',
            Dir::Left => 'L',
            Dir::Right => 'R',
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    Horizontal, // left-right. row
    Vertical,   // up-down. col
}

impl Axis {
    pub fn iter() -> impl Iterator<Item = Axis> {
        [Axis::Horizontal, Axis::Vertical].iter().cloned()
    }

    pub fn as_usize(&self) -> usize {
        match &self {
            Axis::Horizontal => 0,
            Axis::Vertical => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineIndex {
    pub axis: Axis,
    pub index: u8, // index of lines. row if horizontal.
}

impl LineIndex {
    pub fn iter() -> impl Iterator<Item = LineIndex> {
        Axis::iter()
            .flat_map(move |axis| (0..GRID_LEN as u8).map(move |index| LineIndex { axis, index }))
    }

    pub fn choose<R: Rng>(rng: &mut R) -> LineIndex {
        let is_h = rng.gen::<bool>();
        let index = rng.gen_range(0, GRID_LEN as u8);
        LineIndex {
            axis: if is_h {
                Axis::Horizontal
            } else {
                Axis::Vertical
            },
            index,
        }
    }
}

impl LineIndex {
    pub fn new(axis: Axis, index: u8) -> Self {
        assert!((index as usize) < GRID_LEN);
        LineIndex { axis, index }
    }
    pub fn from_move(p: Pos, d: Dir) -> Self {
        assert!(p.move_to(d).is_some(), "{:?} moving {:?}", p, d);
        match d {
            Dir::Up | Dir::Down => LineIndex {
                axis: Axis::Vertical,
                index: p.c,
            },
            Dir::Left | Dir::Right => LineIndex {
                axis: Axis::Horizontal,
                index: p.r,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeIndex {
    pub line: LineIndex,
    pub x: u8, // index within a single line. col if horizontal.
}

impl EdgeIndex {
    pub fn new(line: LineIndex, x: u8) -> Self {
        assert!(x as usize + 1 < GRID_LEN);
        EdgeIndex { line, x }
    }

    pub fn from_move(p: Pos, d: Dir) -> Self {
        assert!(p.move_to(d).is_some(), "{:?} moving {:?}", p, d);
        let line = LineIndex::from_move(p, d);
        let x = match d {
            Dir::Up => p.r - 1,
            Dir::Down => p.r,
            Dir::Left => p.c - 1,
            Dir::Right => p.c,
        };
        EdgeIndex { line, x }
    }
}

#[derive(Debug, Clone)]
pub struct Query {
    pub src: Pos,
    pub dest: Pos,
}

impl Query {
    pub fn height(&self) -> u8 {
        ((self.src.r as i8) - (self.dest.r as i8)).abs() as u8
    }

    pub fn width(&self) -> u8 {
        ((self.src.c as i8) - (self.dest.c as i8)).abs() as u8
    }
}

#[derive(Debug, Clone)]
pub struct GridLines<T: Copy>([[T; GRID_LEN]; 2]);

impl<T: Copy> GridLines<T> {
    pub fn new(value: T) -> GridLines<T> {
        GridLines([[value; GRID_LEN]; 2])
    }
}

impl<T: Copy> Index<LineIndex> for GridLines<T> {
    type Output = T;

    fn index(&self, index: LineIndex) -> &Self::Output {
        &self.0[index.axis.as_usize()][index.index as usize]
    }
}

impl<T: Copy> IndexMut<LineIndex> for GridLines<T> {
    fn index_mut(&mut self, index: LineIndex) -> &mut Self::Output {
        &mut self.0[index.axis.as_usize()][index.index as usize]
    }
}

#[derive(Clone)]
pub struct GridGraph<T: Copy>(GridLines<[T; GRID_LEN - 1]>);

impl<T: Copy> GridGraph<T> {
    pub fn new(value: T) -> GridGraph<T> {
        GridGraph(GridLines::new([value; GRID_LEN - 1]))
    }
}

impl<T: Copy + Default> GridGraph<T> {
    pub fn from_arrays(
        horizontal: [[T; GRID_LEN - 1]; GRID_LEN],
        vertical: [[T; GRID_LEN]; GRID_LEN - 1],
    ) -> GridGraph<T> {
        let mut graph = GridGraph::new(T::default());
        for axis in Axis::iter() {
            for idx in 0..GRID_LEN {
                let line = LineIndex::new(axis, idx as u8);
                for x in 0..GRID_LEN - 1 {
                    let edge = EdgeIndex::new(line, x as u8);
                    let value = match axis {
                        Axis::Horizontal => horizontal[idx][x],
                        Axis::Vertical => vertical[x][idx],
                    };
                    graph[edge] = value;
                }
            }
        }
        graph
    }
}

impl<T: Copy> Index<EdgeIndex> for GridGraph<T> {
    type Output = T;

    fn index(&self, index: EdgeIndex) -> &Self::Output {
        &self.0[index.line][index.x as usize]
    }
}

impl<T: Copy> IndexMut<EdgeIndex> for GridGraph<T> {
    fn index_mut(&mut self, index: EdgeIndex) -> &mut Self::Output {
        &mut self.0[index.line][index.x as usize]
    }
}

#[derive(PartialOrd, PartialEq)]
pub struct UnwrapOrd<T: PartialOrd + PartialEq>(pub T);

impl<T: PartialOrd + PartialEq> Eq for UnwrapOrd<T> {}

impl<T: PartialOrd + PartialEq> Ord for UnwrapOrd<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}
