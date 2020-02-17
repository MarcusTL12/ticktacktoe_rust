use std::{error::Error, io::Write};

use termion::{
    clear, color, cursor,
    event::{Event, Key, MouseButton, MouseEvent},
    style,
};

use termion_game_engine::{GameObject, TerminalGameStatic};
use termion_game_engine_util::{box_mix, Button, TextLabel};

struct Board {
    board: Vec<Vec<Option<bool>>>,
    turn: bool,
    pos: (u16, u16),
    n: usize,
    cols: [Vec<u8>; 2],
}

impl Board {
    fn new(pos: (u16, u16), n: usize) -> Result<Self, Box<dyn Error>> {
        Ok(Board {
            board: (0..n).map(|_| (0..n).map(|_| None).collect()).collect(),
            turn: false,
            pos: pos,
            n: n,
            cols: [
                termion_game_engine::col2fg_str(color::Red)?,
                termion_game_engine::col2fg_str(color::Green)?,
            ],
        })
    }
    //
    fn getcell(&self, mpos: (u16, u16)) -> Option<(u16, u16)> {
        let x = mpos.0 as i16 - self.pos.0 as i16;
        let y = mpos.1 as i16 - self.pos.1 as i16;
        if x < 0
            || x >= (self.n * 4) as i16
            || y < 0
            || y >= (self.n * 2) as i16
            || x % 4 == 0
            || y % 2 == 0
        {
            None
        } else {
            Some((x as u16 / 4, y as u16 / 2))
        }
    }
    //
    fn winner(&self) -> Option<bool> {
        // let temp = self.board.iter().map(|row| {
        //     let counter = row.iter().fold(0, |c, &v| {
        //         if let Some(v) = v {
        //             c + (v as i32 * 2 - 1)
        //         } else {
        //             c
        //         }
        //     });
        //     if counter == self.n as i32 {
        //         Some(true)
        //     } else if -counter == self.n as i32 {
        //         Some(false)
        //     } else {
        //         None
        //     }
        // });
        //
        let temp = (0..self.n)
            .map(|y| {
                (0..self.n).map(move |x| (x, y)).fold(0, |mut c, (x, y)| {
                    if let Some(v) = self.board[y][x] {
                        c += v as i32;
                    }
                    c
                })
            })
            .find(|v| v.abs() as usize == self.n);
        if let Some(t) = temp {
            Some(t < 0)
        } else {
            None
        }
    }
}

impl GameObject for Board {
    fn input(&mut self, e: &Event) {
        match e {
            Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y)) => {
                if let Some(cell) = self.getcell((*x, *y)) {
                    if self.board[cell.1 as usize][cell.0 as usize].is_none() {
                        self.board[cell.1 as usize][cell.0 as usize] =
                            Some(self.turn);
                        self.turn = !self.turn;
                    }
                }
            }
            _ => (),
        }
    }
    //
    fn render(&mut self, buff: &mut Vec<u8>) -> Result<(), Box<dyn Error>> {
        buff.extend(self.cols[self.turn as usize].iter());
        write!(
            buff,
            "{}{}Turn: {}{}{}{}",
            cursor::Goto(self.pos.0, self.pos.1 - 1),
            style::Bold,
            if self.turn { 'O' } else { 'X' },
            cursor::Goto(self.pos.0, self.pos.1),
            color::Fg(color::Reset),
            style::Reset,
        )?;
        //
        for y in 0..self.n {
            let top = y == 0;
            let bottom = y == self.n + 1;
            for x in 0..self.n {
                let left = x == 0;
                write!(
                    buff,
                    "{}",
                    box_mix([
                        Some(top),
                        if !left { Some(top) } else { None },
                        if !top { Some(left) } else { None },
                        Some(left)
                    ])
                )?;
                write!(
                    buff,
                    "{0}{0}{0}",
                    box_mix([Some(top), Some(top), None, None])
                )?;
            }
            write!(
                buff,
                "{}",
                box_mix([
                    None,
                    Some(top),
                    if !top { Some(true) } else { None },
                    Some(!bottom)
                ])
            )?;
            write!(buff, "\n{}", cursor::Left((self.n * 4 + 1) as u16))?;
            for x in 0..self.n {
                let left = x == 0;
                write!(
                    buff,
                    "{}",
                    box_mix([None, None, Some(left), Some(left)])
                )?;
                if let Some(v) = self.board[y][x] {
                    buff.extend(self.cols[v as usize].iter());
                }
                write!(
                    buff,
                    " {}{} ",
                    if let Some(v) = self.board[y][x] {
                        ['X', 'O'][v as usize]
                    } else {
                        ' '
                    },
                    color::Fg(color::Reset)
                )?;
            }
            write!(buff, "{}", box_mix([None, None, Some(true), Some(true)]))?;
            write!(buff, "\n{}", cursor::Left((self.n * 4 + 1) as u16))?;
        }
        for x in 0..self.n {
            let left = x == 0;
            write!(
                buff,
                "{}",
                box_mix([
                    Some(true),
                    if !left { Some(true) } else { None },
                    Some(left),
                    None
                ])
            )?;
            write!(
                buff,
                "{0}{0}{0}",
                box_mix([Some(true), Some(true), None, None])
            )?;
        }
        write!(buff, "{}", box_mix([None, Some(true), Some(true), None]))?;
        Ok(())
    }
}

struct TicTacToe {
    dbuff: Vec<u8>,
    running: bool,
    exitbutton: Button,
    exitlabel: TextLabel,
    board: Board,
    temp: usize,
}

impl TicTacToe {
    fn new() -> Result<Self, Box<dyn Error>> {
        Ok(TicTacToe {
            dbuff: Vec::new(),
            running: true,
            exitbutton: Button::new((1, 1), (4, 2), color::Red)?,
            exitlabel: TextLabel::new(
                (5, 2),
                String::from("<- Exit"),
                color::Red,
            )?,
            board: Board::new((1, 5), 3)?,
            temp: 0,
        })
    }
}

impl TerminalGameStatic for TicTacToe {
    fn update(
        &mut self,
        e: Event,
        buff: &mut Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        self.exitbutton.input(&e);
        self.board.input(&e);
        //
        match e {
            Event::Key(Key::Char(' ')) => self.temp += 1,
            _ => (),
        }
        //
        if self.exitbutton.released(MouseButton::Left) {
            self.running = false;
        }
        //
        if let Some(_) = self.board.winner() {
            self.running = false;
        }
        //
        write!(
            buff,
            "{}{}{}{}",
            clear::All,
            style::Reset,
            color::Bg(color::Reset),
            color::Fg(color::White),
        )
        .unwrap();
        //
        self.exitbutton.render(buff)?;
        self.exitlabel.render(buff)?;
        self.board.render(buff)?;
        //
        if let Ok((_, h)) = termion::terminal_size() {
            write!(buff, "{}", cursor::Goto(1, h)).unwrap();
            buff.append(&mut self.dbuff);
        }
        Ok(())
    }
    //
    fn running(&self) -> bool {
        self.running
    }
}

fn main() {
    let mut game = TicTacToe::new().expect("Game to load");
    game.start().expect("Game to start");
}
