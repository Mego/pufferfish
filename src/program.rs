use std::{
    io::{self, Read},
    ops::{Add, AddAssign, Index},
    process::exit,
};

use bounded_integer::bounded_integer;
use divisors_fixed::Divisors;
use grid::Grid;
use rand::{prelude::*, rng};

use crate::parser::{parse_names, populate_tanks};

bounded_integer! {
    struct IpRow(0, 4);
}

bounded_integer! {
    struct IpCol(0, 3);
}

bounded_integer! {
    #[allow(dead_code)]
    enum CycleInstruction {
        Subtract,
        Swap,
        Dup,
        Drop
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct InstructionPointer(IpRow, IpCol);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Up,
    Left,
    Right,
    Down,
}

impl InstructionPointer {
    pub fn move_dir(self, rhs: Direction) -> Self {
        match rhs {
            Direction::Up => Self(self.0.wrapping_sub(1), self.1),
            Direction::Down => Self(self.0.wrapping_add(1), self.1),
            Direction::Left => Self(self.0, self.1.wrapping_sub(1)),
            Direction::Right => Self(self.0, self.1.wrapping_add(1)),
        }
    }
}

impl Default for InstructionPointer {
    fn default() -> Self {
        Self(IpRow::const_new::<0>(), IpCol::const_new::<0>())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Tank {
    pub(crate) grid: Grid<usize>,
    name: String,
    cycle_instr: CycleInstruction,
    acc: usize,
}

impl Tank {
    pub(crate) fn new(name: String, grid: Grid<usize>) -> Self {
        Self {
            grid,
            name,
            cycle_instr: Default::default(),
            acc: Default::default(),
        }
    }
}

impl Add for Tank {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign for Tank {
    fn add_assign(&mut self, rhs: Self) {
        self.grid.indexed_iter_mut().for_each(|(i, x)| {
            *x += rhs.grid[i];
        });
    }
}

impl Index<InstructionPointer> for Tank {
    type Output = usize;

    fn index(&self, index: InstructionPointer) -> &Self::Output {
        &self.grid[(index.0.into(), index.1.into())]
    }
}

pub struct Program {
    aquarium: Grid<Tank>,
    ftp: (usize, usize),
    ip: InstructionPointer,
    ip_dir: Direction,
    stack: Vec<isize>,
    trampoline_set: bool,
}

impl Program {
    pub(crate) fn build_aquarium(tanks: Vec<Tank>) -> Self {
        let n = tanks.len();
        let sqrt_n = (n as f64).sqrt();
        let height = n
            .divisors_unordered()
            .into_iter()
            .min_by(|&a, &b| (((a as f64) - sqrt_n).abs()).total_cmp(&((b as f64) - sqrt_n).abs()))
            .unwrap();
        let width = n / height;
        Self {
            aquarium: Grid::from_vec(tanks, width),
            ftp: (0, 0),
            ip: Default::default(),
            ip_dir: Direction::Right,
            stack: Default::default(),
            trampoline_set: false,
        }
    }

    pub fn new(code: &str) -> Result<Self, anyhow::Error> {
        let names = parse_names(code)?;
        let tanks = populate_tanks(names)?;
        Ok(Self::build_aquarium(tanks))
    }

    fn update_ip(&mut self) {
        self.ip = self.ip.move_dir(self.ip_dir);
    }

    fn down(&mut self) {
        self.ip_dir = Direction::Down;
        self.update_ip();
    }

    fn up(&mut self) {
        self.ip_dir = Direction::Up;
        self.update_ip();
    }

    fn right(&mut self) {
        self.ip_dir = Direction::Right;
        self.update_ip();
    }

    fn left(&mut self) {
        self.ip_dir = Direction::Left;
        self.update_ip();
    }

    fn push_acc(&mut self) {
        let tank = &mut self.aquarium[self.ftp];
        self.stack.push(tank.acc as isize);
        tank.acc += 1;
    }

    fn cycle_sub(&mut self) {
        assert!(self.stack.len() >= 2);
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(a - b);
    }

    fn cycle_swap(&mut self) {
        assert!(self.stack.len() >= 2);
        let last = self.stack.len() - 1;
        self.stack.swap(last, last - 1);
    }

    fn cycle_dup(&mut self) {
        assert!(!self.stack.is_empty());
        let &a = self.stack.last().unwrap();
        self.stack.push(a);
    }

    fn cycle_drop(&mut self) {
        self.stack.pop();
    }

    fn cycle(&mut self) {
        match self.aquarium[self.ftp].cycle_instr {
            CycleInstruction::Subtract => self.cycle_sub(),
            CycleInstruction::Drop => self.cycle_drop(),
            CycleInstruction::Dup => self.cycle_dup(),
            CycleInstruction::Swap => self.cycle_swap(),
        }
        self.aquarium[self.ftp].cycle_instr += 1;
        self.update_ip();
    }

    fn tunnel(&mut self) {
        if let Some(&a) = self.stack.last()
            && a > 0
        {
            self.trampoline_set = false;
        } else {
            self.trampoline_set = true;
        }
        self.update_ip();
    }

    fn hop(&mut self) {
        match self.ip_dir {
            Direction::Down => {
                self.ftp.0 += 1;
                self.ftp.0 %= self.aquarium.rows();
            }
            Direction::Up => {
                self.ftp.0 = self
                    .ftp
                    .0
                    .checked_sub(1)
                    .unwrap_or(self.aquarium.rows() - 1);
            }
            Direction::Left => {
                self.ftp.1 = self
                    .ftp
                    .1
                    .checked_sub(1)
                    .unwrap_or(self.aquarium.cols() - 1);
            }
            Direction::Right => {
                self.ftp.1 += 1;
                self.ftp.1 %= self.aquarium.cols();
            }
        }
    }

    fn call(&mut self) {
        match self.aquarium[self.ftp].name.chars().next().unwrap() {
            'e' => exit(0),
            'i' => {
                let mut buf = [0u8; 1];
                if let Ok(n) = io::stdin().read(&mut buf)
                    && n == 0
                {
                    self.stack.push(-1);
                } else {
                    self.stack.push(buf[0] as isize);
                }
            }
            'o' => {
                let val = self.stack.pop().unwrap();
                let s = String::from_utf8_lossy(&val.to_be_bytes()).to_string();
                print!("{s}");
            }
            'y' => {
                let mut rng = rng();
                self.ip_dir = *[
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                    Direction::Up,
                ]
                .choose(&mut rng)
                .unwrap();
            }
            _ => unimplemented!(),
        }
        self.update_ip();
    }

    pub fn step(&mut self) {
        let instr = self.aquarium[self.ftp][self.ip] % 10;
        match instr {
            0 => {
                self.update_ip();
            }
            _ if self.trampoline_set => {
                self.trampoline_set = false;
                self.update_ip();
            }
            1 => {
                self.down();
            }
            2 => {
                self.up();
            }
            3 => {
                self.right();
            }
            4 => {
                self.left();
            }
            5 => {
                self.push_acc();
            }
            6 => {
                self.cycle();
            }
            7 => {
                self.tunnel();
            }
            8 => {
                self.hop();
            }
            9 => {
                self.call();
            }
            _ => unreachable!(),
        }
    }
}
