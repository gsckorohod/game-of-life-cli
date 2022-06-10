extern crate termion;

use termion::event::{Key, Event};
use termion::input::TermRead;
use termion::async_stdin;
use termion::raw::IntoRawMode;
use std::io::{Read, Write, stdout, stdin};
use std::thread::sleep;
use std::time::Duration;
use std::cmp::max;
use std::collections::VecDeque;
use std::result;


const DEAD: &str = "  ";
const ALIVE: &str = "██";
const CORNERS: [char; 4] = ['╔', '╗', '╝', '╚'];
const BORDER_H: &str = "══";
const BORDER_V: char = '║';
const SELECTED_DEAD: &str = "░░";
const SELECTED_ALIVE: &str = "▒▒";
const HISTORY_LEN: usize = 20;


fn write_title(stdout: &mut dyn Write, write_help: bool) {
    write!(stdout, "\r{}\n\r", "Game Of Life");
    write!(stdout, "{}\n\r", "------------");
    if write_help {
        write!(stdout, "{}\n\r", "Controls:");
        write!(stdout, "{}\n\r", "* Arrow keys - move cursor");
        write!(stdout, "{}\n\r", "* Space - toggle cell");
        write!(stdout, "{}\n\r", "* R/S - [R]un / [S]top");
        write!(stdout, "{}\n\r", "* P/N - [P]rev/[N]ext");
        write!(stdout, "{}\n\r", "        (Single Step)");
        write!(stdout, "{}\n\r", "* C - [C]lear");
        write!(stdout, "{}\n\r", "* T - [T]oggle cursor");
        write!(stdout, "{}\n\r", "* E - [E]dit settings");
        write!(stdout, "{}\n\r", "------------");
    }
}


pub struct Universe {
    width: usize,
    height: usize,
    cells: Vec<bool>,
    selected_cell: (usize, usize),
    show_cursor: bool,
    is_running: bool,
    history: VecDeque<Vec<bool>>,
    should_write_help: bool,
}


impl Universe {
    pub fn new(width: usize, height: usize) -> Universe {
        Universe {
            width: width,
            height: height,
            cells: vec![false; width * height],
            selected_cell: (0, 0),
            show_cursor: false,
            is_running: false,
            history: VecDeque::new(),
            should_write_help: true,
        }
    }

    fn get_index(&self, row: usize, column: usize) -> usize {
        (row * self.width + column) as usize
    }

    pub fn set_cells(&mut self, cells: &[(usize, usize)]) {
        for (row, col) in cells {
            let idx = self.get_index(*row, *col);
            self.cells[idx] = true;
        }
    }

    pub fn render(&self, stdout: &mut dyn Write) {
        write!(stdout,
           "{}{}{}",
           termion::cursor::Goto(1, 1),
           termion::clear::All,
           termion::cursor::Hide);
        write_title(stdout, self.should_write_help);

        write!(stdout, "{}", CORNERS[0]);
        for i in 0..self.width {write!(stdout, "{}", BORDER_H);}
        write!(stdout, "{}\n\r", CORNERS[1]);

        for i in 0..self.height {
            write!(stdout, "{}", BORDER_V);
            for j in 0..self.width {
                let ind = self.get_index(i, j);

                if self.cells[ind] {
                    if ((i, j) == self.selected_cell) && self.show_cursor {
                        write!(stdout, "{}", SELECTED_ALIVE);
                    } else {write!(stdout, "{}", ALIVE);}
                } else {
                    if ((i, j) == self.selected_cell) && self.show_cursor {
                        write!(stdout, "{}", SELECTED_DEAD);
                    } else {write!(stdout, "{}", DEAD);}
                }
            }
            write!(stdout, "{}\n\r", BORDER_V);
        }

        write!(stdout, "{}", CORNERS[3]);
        for i in 0..self.width {write!(stdout, "{}", BORDER_H);}
        write!(stdout, "{}\n\r", CORNERS[2]);
    }

    pub fn move_cursor(&mut self, r: isize, c: isize) {
        if r < 0 {
            if r.abs() as usize > self.selected_cell.0 {
                self.selected_cell.0 = self.height - ((r.abs() as usize - self.selected_cell.0) % self.height);
            } else {self.selected_cell.0 -= r.abs() as usize;}
        } else {
            self.selected_cell.0 = (self.selected_cell.0 + r as usize) % self.height;
        }

        if c < 0 {
            if c.abs() as usize > self.selected_cell.1 {
                self.selected_cell.1 = self.width - ((c.abs() as usize - self.selected_cell.1) % self.width);
            } else {self.selected_cell.1 -= c.abs() as usize;}
        } else {
            self.selected_cell.1 = (self.selected_cell.1 + c as usize) % self.width;
        }
    }

    fn is_in_bounds(&self, row: isize, col: isize) -> bool {
        return (row >= 0) &&
               (row < self.height as isize) &&
               (col >= 0) &&
               (col < self.width as isize);
    }

    pub fn get_cell(&self, row: usize, col: usize) -> bool {
        let ind = self.get_index(row, col);
        return self.cells[ind];
    }

    pub fn set_cell(&mut self, row: usize, col: usize, val: bool) {
        let ind = self.get_index(row, col);
        self.cells[ind] = val;
    }

    fn live_neighbour_count(&self, row: usize, col: usize) -> usize {
        let mut ans: usize = 0;
        let row = row as isize;
        let col = col as isize;

        for r in (row - 1)..=(row + 1) {
            for c in (col - 1)..=(col + 1) {
                if self.is_in_bounds(r, c) && !((r, c) == (row, col)){
                    ans += self.get_cell(r as usize, c as usize) as usize;
                }
            }
        }

        return ans;
    }

    pub fn tick(&mut self) {
        let mut next = vec![false; self.width * self.height];
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbours = self.live_neighbour_count(row, col);
                next[idx] = match (cell, live_neighbours) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (otherwise, _) => otherwise,
                };
            }
        }
        if self.history.len() >= HISTORY_LEN {self.history.pop_front();}
        self.history.push_back(self.cells.clone());
        self.cells = next;
    }

    pub fn tick_back(&mut self) -> Result<&str, &str> {
        match self.history.pop_back() {
            Some(x) => {self.cells = x},
            None => {return Err("No more moves in history!");},
        };
        return Ok("Returned to previous step");
    }

    pub fn toggle_selected_cell(&mut self) {
        self.set_cell(self.selected_cell.0,
                      self.selected_cell.1,
                      !self.get_cell(self.selected_cell.0,
                                    self.selected_cell.1,));
    }

    pub fn clear(&mut self) {
        self.cells = vec![false; self.width * self.height];
    }
}


fn main() {
    let stdin = async_stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut it = stdin.keys();

    let mut game = Universe::new(6, 6);
    game.show_cursor = true;
    game.render(&mut stdout);

    let mut tick_millis: u64 = 200;

    loop {
        sleep(Duration::from_millis(1));
        let b = it.next();

        match b {
            Some(x) => match x.unwrap() {
                Key::Up => {
                    game.move_cursor(-1, 0);
                    game.render(&mut stdout);
                    stdout.flush().unwrap();
                }
                Key::Down => {
                    game.move_cursor(1, 0);
                    game.render(&mut stdout);
                    stdout.flush().unwrap();
                }
                Key::Right => {
                    game.move_cursor(0, 1);
                    game.render(&mut stdout);
                    stdout.flush().unwrap();
                }
                Key::Left => {
                    game.move_cursor(0, -1);
                    game.render(&mut stdout);
                    stdout.flush().unwrap();
                }
                Key::Char('r') => {
                    game.is_running = true;
                }
                Key::Char('s') => {
                    game.is_running = false;
                }
                Key::Char('n') => {
                    game.tick();
                    game.render(&mut stdout);
                    stdout.flush().unwrap();
                }
                Key::Char('p') => {
                    match game.tick_back() {
                        Ok(_) => {game.render(&mut stdout);}
                        Err(msg) => {write!(stdout, "\r{}", msg).unwrap();}
                    };
                    stdout.flush().unwrap();
                }
                Key::Char('c') => {
                    game.clear();
                    game.render(&mut stdout);
                    stdout.flush().unwrap();
                }
                Key::Char('t') => {
                    game.show_cursor = !game.show_cursor;
                    game.render(&mut stdout);
                    stdout.flush().unwrap();
                }
                Key::Char(' ') => {
                    game.toggle_selected_cell();
                    game.render(&mut stdout);
                    stdout.flush().unwrap();
                }
                Key::Char('-') => {tick_millis += 50;}
                Key::Char('+') => {tick_millis = max(tick_millis - 50, 50);}
                Key::Char('q') => break,
                other => {
                    write!(stdout, "Unexpected key: {:?}", other).unwrap();
                    stdout.flush().unwrap();
                }
            }
            _ => {}
        }

        if game.is_running {
            game.tick();
            game.render(&mut stdout);
            // write!(stdout, "{}", "game was updated by regular tick").unwrap();
            stdout.flush().unwrap();
            sleep(Duration::from_millis(tick_millis));
        }
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
