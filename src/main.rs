extern crate termion;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};


const DEAD: &str = "  ";
const ALIVE: &str = "██";
const CORNERS: [char; 4] = ['╔', '╗', '╝', '╚'];
const BORDER_H: &str = "══";
const BORDER_V: char = '║';
const SELECTED_DEAD: &str = "░░";
const SELECTED_ALIVE: &str = "▒▒";


fn write_title(stdout: &mut dyn Write) {
    write!(stdout, "\r{}\n\r", "Game Of Life");
    write!(stdout, "{}\n\r", "------------");
    write!(stdout, "{}\n\r", "Controls:");
    write!(stdout, "{}\n\r", "* Arrow keys - move cursor");
    write!(stdout, "{}\n\r", "* Space - toggle cell");
    write!(stdout, "{}\n\r", "* R/S - [R]un / [S]top");
    write!(stdout, "{}\n\r", "* P/N - [P]rev/[N]ext");
    write!(stdout, "{}\n\r", "        (Single Step)");
    write!(stdout, "{}\n\r", "* C - [C]lear");
    write!(stdout, "{}\n\r", "------------");
}


pub struct Universe {
    width: usize,
    height: usize,
    cells: Vec<bool>,
    selected_cell: (usize, usize),
    show_cursor: bool,
}


impl Universe {
    pub fn new(width: usize, height: usize) -> Universe {
        Universe {
            width: width,
            height: height,
            cells: vec![false; width * height],
            selected_cell: (0, 0),
            show_cursor: false,
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
           "{}{}",
           termion::cursor::Goto(1, 1),
           termion::clear::All);
        write_title(stdout);

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
        self.cells = next;
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
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut game = Universe::new(20, 20);
    game.show_cursor = true;
    game.render(&mut stdout);

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Up => {
                game.move_cursor(-1, 0);
                game.render(&mut stdout);
            }
            Key::Down => {
                game.move_cursor(1, 0);
                game.render(&mut stdout);
            }
            Key::Right => {
                game.move_cursor(0, 1);
                game.render(&mut stdout);
            }
            Key::Left => {
                game.move_cursor(0, -1);
                game.render(&mut stdout);
            }
            Key::Char('n') => {
                game.tick();
                game.render(&mut stdout);
            }
            Key::Char('c') => {
                game.clear();
                game.render(&mut stdout);
            }
            Key::Char(' ') => {
                game.toggle_selected_cell();
                game.render(&mut stdout);
            }
            Key::Char('q') => break,
            _ => {}
        }
        stdout.flush().unwrap();
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();

}
