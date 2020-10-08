mod history;
mod should_execute;

use history::History;
use should_execute::should_execute;

use crossterm::{cursor, event, execute, queue, style, terminal};
use std::io::prelude::*;

pub struct Repl {
    history: History,
    leader: &'static str,
    leader_len: usize,
    continued_leader: &'static str,
    continued_leader_len: usize,
}

impl Repl {
    pub fn new(leader: &'static str, continued_leader: &'static str) -> Self {
        Self {
            history: History::new(),
            leader,
            leader_len: leader.chars().count(),
            continued_leader,
            continued_leader_len: leader.chars().count(),
        }
    }

    pub fn with_history_capacity(
        leader: &'static str,
        continued_leader: &'static str,
        capacity: usize,
    ) -> Self {
        Self {
            history: History::with_capacity(capacity),
            leader,
            leader_len: leader.chars().count(),
            continued_leader,
            continued_leader_len: leader.chars().count(),
        }
    }

    fn cur_str<'a>(&'a self, cursor: &Cursor, lines: &'a Vec<String>) -> &'a str {
        &(if cursor.use_history {
            // unwrap because if use_history is enabled, there must be at least one element in
            // history
            self.history.cur().unwrap()
        } else {
            lines
        })[cursor.lineno]
    }

    fn replace_with_history(&self, lines: &mut Vec<String>) {
        let cur = self.history.cur().unwrap();
        lines.resize(cur.len(), String::new());

        for (i, string) in cur.iter().enumerate() {
            lines[i].clear();
            lines[i] += string;
        }
    }

    fn print_lines(
        &self,
        stdout: &mut std::io::Stdout,
        c: &mut Cursor,
        lines: &Vec<String>,
    ) -> crossterm::Result<()> {
        queue!(
            stdout,
            cursor::MoveUp(c.lineno as u16),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        )?;
        let mut is_first = true;

        for line in lines {
            let (leader, leader_len) = if is_first {
                is_first = false;
                (self.leader, self.leader_len)
            } else {
                (self.continued_leader, self.continued_leader_len)
            };

            queue!(
                stdout,
                style::Print(leader),
                style::ResetColor,
                style::Print(line),
                cursor::MoveToColumn((leader_len + c.charno + 1) as u16)
            )?;
        }

        let leader_len = if c.lineno == 0 {
            self.leader_len
        } else {
            self.continued_leader_len
        };

        c.charno = std::cmp::min(c.charno, lines[c.lineno].len());

        execute!(
            stdout,
            cursor::MoveUp((lines.len() - c.lineno) as u16),
            cursor::MoveToColumn((leader_len + c.charno) as u16)
        )
    }

    pub fn next(&mut self, colour: style::Color) -> crossterm::Result<String> {
        let mut stdout = std::io::stdout();
        let mut lines = Vec::new();
        lines.push(String::new());

        let mut c = Cursor::default();

        terminal::enable_raw_mode()?;

        execute!(
            stdout,
            style::SetForegroundColor(colour),
            style::Print(self.leader),
            style::ResetColor
        )?;

        loop {
            let s = match event::read()? {
                event::Event::Key(e) => match e.code {
                    event::KeyCode::Char(chr)
                        if e.modifiers.contains(event::KeyModifiers::CONTROL) && chr == 'c' =>
                    {
                        terminal::disable_raw_mode()?;
                        println!();
                        std::process::exit(0);
                    }
                    event::KeyCode::Char(chr) => {
                        if c.use_history {
                            self.replace_with_history(&mut lines);
                            c.use_history = false;
                        };

                        lines[c.lineno].insert(c.charno, chr);
                        c.charno += 1;

                        &lines[c.lineno] as &str
                    }

                    event::KeyCode::Left if c.charno > 0 => {
                        c.charno -= 1;
                        self.cur_str(&c, &lines)
                    }
                    event::KeyCode::Right => {
                        let s = self.cur_str(&c, &lines);

                        if c.charno < s.chars().count() {
                            c.charno += 1;
                        };

                        s
                    }
                    event::KeyCode::Down => {
                        let s = match self.history.next() {
                            Some(s) => &s[c.lineno],
                            None => {
                                c.use_history = false;
                                &lines[c.lineno]
                            }
                        };

                        let s_len = s.chars().count();

                        if c.charno == 0 || c.charno > s_len {
                            c.charno = s.chars().count();
                        }
                        s
                    }
                    event::KeyCode::Up => {
                        c.use_history = true;

                        let s = match self.history.prev() {
                            Some(s) => &s[c.lineno],
                            None => match self.history.cur() {
                                Some(s) => &s[c.lineno],
                                None => {
                                    c.use_history = false;
                                    &lines[c.lineno]
                                }
                            },
                        };

                        let s_len = s.chars().count();

                        if c.charno == 0 || c.charno > s_len {
                            c.charno = s_len;
                        }
                        s
                    }

                    event::KeyCode::Backspace if c.charno > 0 => {
                        if c.use_history {
                            self.replace_with_history(&mut lines);
                            c.use_history = false;
                        };

                        c.charno -= 1;
                        lines[c.lineno].remove(c.charno);
                        &lines[c.lineno]
                    }
                    event::KeyCode::Delete => {
                        let s = self.cur_str(&c, &lines);
                        if c.charno < s.chars().count() {
                            if c.use_history {
                                self.replace_with_history(&mut lines);
                                c.use_history = false;
                            };

                            lines[c.lineno].remove(c.charno);
                        }
                        &lines[c.lineno]
                    }

                    event::KeyCode::Enter => {
                        if lines.len() == 1 {
                            match &lines[0] as &str {
                                "exit" => {
                                    terminal::disable_raw_mode()?;
                                    println!();
                                    std::process::exit(0);
                                }
                                "clear" => {
                                    c.charno = 0;
                                    lines[0].clear();

                                    execute!(
                                        stdout,
                                        terminal::Clear(terminal::ClearType::All),
                                        cursor::MoveTo(0, 0),
                                        style::SetForegroundColor(colour),
                                        style::Print(self.leader),
                                        style::ResetColor,
                                    )?;

                                    continue;
                                }
                                _ => {}
                            }
                        }
                        if should_execute(&lines) {
                            break;
                        } else {
                            c.lineno += 1;
                            c.charno = 0;
                            lines.insert(c.lineno, String::new());
                            execute!(stdout, cursor::MoveToNextLine(1))?;
                            ""
                        }
                    }
                    _ => self.cur_str(&c, &lines),
                },
                _ => self.cur_str(&c, &lines),
            };

            queue!(
                stdout,
                terminal::Clear(terminal::ClearType::CurrentLine),
                cursor::MoveToColumn(0),
                style::SetForegroundColor(colour),
            )?;

            let (leader, leader_len) = if c.lineno == 0 {
                (self.leader, self.leader_len)
            } else {
                (self.continued_leader, self.continued_leader_len)
            };

            execute!(
                stdout,
                style::Print(leader),
                style::ResetColor,
                style::Print(s),
                cursor::MoveToColumn((leader_len + c.charno + 1) as u16)
            )?;
        }

        terminal::disable_raw_mode()?;
        println!();

        let src = if c.use_history {
            Ok(self.history.cur().unwrap().join("\n"))
        } else {
            Ok(lines.join("\n"))
        };

        self.history.push(lines);

        src
    }
}

#[derive(Default)]
struct Cursor {
    use_history: bool,
    lineno: usize,
    charno: usize,
}
