mod history;

use history::History;

use crossterm::{cursor, event, execute, style, terminal};
use std::io::prelude::*;

pub struct Repl {
    history: History,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            history: History::new(),
        }
    }

    pub fn with_history_capacity(capacity: usize) -> Self {
        Self {
            history: History::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, string: String) {
        self.history.push(string);
    }

    fn cur_str<'a>(&'a self, use_history: bool, string: &'a String) -> &'a str {
        if use_history {
            // unwrap because if use_history is enabled, there must be at least one element in
            // history
            self.history.cur().unwrap()
        } else {
            string
        }
    }

    pub fn next(&mut self, leader: &str, colour: style::Color) -> crossterm::Result<String> {
        let mut stdout = std::io::stdout();
        let mut string = String::new();
        let mut use_history = false;

        let mut cursor_pos = 0;

        terminal::enable_raw_mode()?;

        execute!(
            stdout,
            style::SetForegroundColor(colour),
            style::Print(leader),
            style::ResetColor
        )?;

        loop {
            let s = match event::read()? {
                event::Event::Key(e) => match e.code {
                    event::KeyCode::Char(c)
                        if e.modifiers.contains(event::KeyModifiers::CONTROL) && c == 'c' =>
                    {
                        terminal::disable_raw_mode()?;
                        std::process::exit(0);
                    }
                    event::KeyCode::Char(c) => {
                        if use_history {
                            string.clear();
                            string += self.history.cur().unwrap();
                            use_history = false;
                        };

                        string.insert(cursor_pos, c);
                        cursor_pos += 1;
                        &string as &str
                    }

                    event::KeyCode::Left if cursor_pos > 0 => {
                        cursor_pos -= 1;
                        self.cur_str(use_history, &string)
                    }
                    event::KeyCode::Right => {
                        let s = self.cur_str(use_history, &string);

                        if cursor_pos < s.chars().count() {
                            cursor_pos += 1;
                        };

                        s
                    }
                    event::KeyCode::Down => {
                        let s = match self.history.next() {
                            Some(s) => s,
                            None => {
                                use_history = false;
                                &string
                            }
                        };

                        let s_len = s.chars().count();

                        if cursor_pos == 0 || cursor_pos > s_len {
                            cursor_pos = s.chars().count();
                        }
                        s
                    }
                    event::KeyCode::Up => {
                        use_history = true;

                        let s = match self.history.prev() {
                            Some(s) => s,
                            None => match self.history.cur() {
                                Some(s) => s,
                                None => {
                                    use_history = false;
                                    &string
                                }
                            },
                        };

                        let s_len = s.chars().count();

                        if cursor_pos == 0 || cursor_pos > s_len {
                            cursor_pos = s_len;
                        }
                        s
                    }

                    event::KeyCode::Backspace if cursor_pos > 0 => {
                        if use_history {
                            string.clear();
                            string += self.history.cur().unwrap();
                            use_history = false;
                        };

                        cursor_pos -= 1;
                        string.remove(cursor_pos);
                        &string
                    }
                    event::KeyCode::Delete => {
                        let s = self.cur_str(use_history, &string);
                        if cursor_pos < s.chars().count() {
                            if use_history {
                                string.clear();
                                string += self.history.cur().unwrap();
                                use_history = false;
                            };

                            string.remove(cursor_pos);
                        }
                        &string
                    }

                    event::KeyCode::Enter => {
                        break;
                    }
                    _ => self.cur_str(use_history, &string),
                },
                _ => self.cur_str(use_history, &string),
            };

            execute!(
                stdout,
                terminal::Clear(terminal::ClearType::CurrentLine),
                cursor::MoveToColumn(0),
                style::SetForegroundColor(colour),
                style::Print(leader),
                style::ResetColor,
                style::Print(s),
                cursor::MoveToColumn((leader.chars().count() + cursor_pos + 1) as u16),
            )?;
        }

        terminal::disable_raw_mode()?;
        println!();

        if use_history {
            Ok(self.history.cur().unwrap().to_owned())
        } else {
            Ok(string)
        }
    }
}
