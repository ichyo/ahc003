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

pub struct ArrayGridGraph<T> {
    pub horizontal: [[T; GRID_LEN - 1]; GRID_LEN],
    pub vertical: [[T; GRID_LEN]; GRID_LEN - 1],
}

impl<T: Copy> ArrayGridGraph<T> {
    pub fn new(value: T) -> ArrayGridGraph<T> {
        ArrayGridGraph {
            horizontal: [[value; GRID_LEN - 1]; GRID_LEN],
            vertical: [[value; GRID_LEN]; GRID_LEN - 1],
        }
    }
    pub fn from_arrays(
        horizontal: [[T; GRID_LEN - 1]; GRID_LEN],
        vertical: [[T; GRID_LEN]; GRID_LEN - 1],
    ) -> ArrayGridGraph<T> {
        ArrayGridGraph {
            horizontal,
            vertical,
        }
    }
}

impl<T> ArrayGridGraph<T> {
    pub fn get(&self, p: Pos, d: Dir) -> &T {
        assert!(p.move_to(d).is_some(), "{:?} moving {:?}", p, d);
        match d {
            Dir::Up => &self.vertical[p.r as usize - 1][p.c as usize],
            Dir::Down => &self.vertical[p.r as usize][p.c as usize],
            Dir::Left => &self.horizontal[p.r as usize][p.c as usize - 1],
            Dir::Right => &self.horizontal[p.r as usize][p.c as usize],
        }
    }

    pub fn get_mut(&mut self, p: Pos, d: Dir) -> &mut T {
        assert!(p.move_to(d).is_some(), "{:?} moving {:?}", p, d);
        match d {
            Dir::Up => &mut self.vertical[p.r as usize - 1][p.c as usize],
            Dir::Down => &mut self.vertical[p.r as usize][p.c as usize],
            Dir::Left => &mut self.horizontal[p.r as usize][p.c as usize - 1],
            Dir::Right => &mut self.horizontal[p.r as usize][p.c as usize],
        }
    }
}

pub trait GridGraph<T> {
    fn get(&self, p: Pos, d: Dir) -> &T;
}

impl<T> GridGraph<T> for ArrayGridGraph<T> {
    fn get(&self, p: Pos, d: Dir) -> &T {
        self.get(p, d)
    }
}
