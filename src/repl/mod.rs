mod history;
mod should_execute;

use history::History;
use should_execute::should_execute;

use crossterm::{cursor, event, execute, queue, style, terminal};
use std::cmp::min;
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

    fn cur<'a>(&'a self, c: &Cursor, lines: &'a Vec<String>) -> &'a Vec<String> {
        if c.use_history {
            // unwrap because if use_history is enabled, there must be at least one element in
            // history
            self.history.cur().unwrap()
        } else {
            lines
        }
    }

    fn cur_str<'a>(&'a self, c: &Cursor, lines: &'a Vec<String>) -> &'a str {
        &self.cur(c, lines)[c.lineno]
    }

    fn replace_with_history(&self, lines: &mut Vec<String>) {
        // TODO check if history iter should be reset
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
        colour: style::Color,
    ) -> crossterm::Result<()> {
        if c.lineno > 0 {
            queue!(stdout, cursor::MoveUp(c.lineno as u16))?;
        }

        queue!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown),)?;
        let mut is_first = true;

        for line in lines {
            let leader = if is_first {
                is_first = false;
                self.leader
            } else {
                self.continued_leader
            };

            queue!(
                stdout,
                cursor::MoveToColumn(0),
                style::SetForegroundColor(colour),
                style::Print(leader),
                style::ResetColor,
                style::Print(line),
                style::Print("\n"),
            )?;
        }

        let leader_len = if c.lineno == 0 {
            self.leader_len
        } else {
            self.continued_leader_len
        };

        c.charno = min(c.charno, lines[c.lineno].len());

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

                    // At the top of the current block, go to previous history block
                    event::KeyCode::Up if c.lineno == 0 => {
                        c.use_history = true;

                        let lines = match self.history.prev() {
                            Some(s) => {
                                self.print_lines(&mut stdout, &mut c, &s, colour)?;
                                c.lineno = s.len() - 1;
                                queue!(stdout, cursor::MoveDown(c.lineno as u16))?;
                                s
                            }
                            None => match self.history.cur() {
                                Some(s) => s,
                                None => {
                                    c.use_history = false;
                                    &lines
                                }
                            },
                        };

                        let s = &lines[c.lineno];
                        let s_len = s.chars().count();

                        if c.charno == 0 || c.charno > s_len {
                            c.charno = s_len;
                        }
                        s
                    }
                    // In the middle of a block, go up one line
                    event::KeyCode::Up => {
                        c.lineno -= 1;
                        queue!(stdout, cursor::MoveUp(1))?;
                        let s = self.cur_str(&c, &lines);
                        c.charno = min(s.len(), c.charno);
                        s
                    }

                    // At the bottom of the block, and in history. This means that there are more
                    // blocks down, either further down the history or when history is over, the
                    // editable lines itself
                    event::KeyCode::Down
                        if c.use_history && (c.lineno + 1) == self.history.cur().unwrap().len() =>
                    {
                        let lines = match self.history.next() {
                            Some(s) => s,
                            None => {
                                c.use_history = false;
                                &lines
                            }
                        };

                        queue!(stdout, cursor::MoveUp(c.lineno as u16))?;
                        c.lineno = 0;
                        self.print_lines(&mut stdout, &mut c, lines, colour)?;

                        let s = &lines[c.lineno];
                        let s_len = s.chars().count();

                        if c.charno == 0 || c.charno > s_len {
                            c.charno = s.chars().count();
                        }
                        s
                    }
                    // When in the end of editable lines, nothing should be done
                    event::KeyCode::Down if !c.use_history && (c.lineno + 1) == lines.len() => {
                        &lines[c.lineno]
                    }
                    // Somewhere in the block, go to next line
                    event::KeyCode::Down => {
                        c.lineno += 1;
                        queue!(stdout, cursor::MoveDown(1))?;
                        let s = self.cur_str(&c, &lines);
                        c.charno = min(s.len(), c.charno);
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
                            execute!(stdout, style::Print("\n"))?;
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

        let src = self.cur(&c, &lines).join("\n");

        self.history.push(lines);

        Ok(src)
    }
}

#[derive(Debug, Default)]
struct Cursor {
    use_history: bool,
    lineno: usize,
    charno: usize,
}
