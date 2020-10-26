mod get_indent;
mod history;
mod linter;
#[macro_use]
mod macros;

use get_indent::get_indent;
use history::History;

use crossterm::{cursor, event, execute, queue, style, terminal};
use std::cmp::min;
use std::io::prelude::*;
use std::path::PathBuf;

pub struct Repl {
    /// The history of commands run
    history: History,
    /// What to print as the prompt
    ///
    /// > <some-code>
    /// ^^- leader
    leader: &'static str,
    /// The number of characters in the leader, it is stored here since getting number of characters
    /// is an O(n) operation for a utf-8 encoded string
    leader_len: usize,
    /// If the command is more than one line long, what to print on subsequent lines
    ///
    /// > <some-code>
    /// . <some-code>
    /// ^^- continued leader
    continued_leader: &'static str,
    /// The number of characters in the continued leader, it is stored here since getting number of
    /// characters is an O(n) operation for a utf-8 encoded string
    continued_leader_len: usize,
    /// Whether to print the AST generated for the command.
    /// See also `core/src/syntax_node/mod.rs``::SyntaxNode::prt`
    pub show_tree: bool,
}

impl Repl {
    pub fn new(
        leader: &'static str,
        continued_leader: &'static str,
        path: Option<PathBuf>,
    ) -> Self {
        let should_persist = path.is_some();

        let mut repl = Self {
            history: History::new(path),
            leader,
            leader_len: leader.chars().count(),
            continued_leader,
            continued_leader_len: leader.chars().count(),
            show_tree: false,
        };

        if should_persist {
            let _ = repl.history.read_from_file();
        }

        repl
    }

    #[allow(dead_code)]
    pub fn with_history_capacity(
        leader: &'static str,
        continued_leader: &'static str,
        capacity: usize,
        path: Option<PathBuf>,
    ) -> Self {
        let should_persist = path.is_some();

        let mut repl = Self {
            history: History::with_capacity(capacity, path),
            leader,
            leader_len: leader.chars().count(),
            continued_leader,
            continued_leader_len: leader.chars().count(),
            show_tree: false,
        };

        if should_persist {
            let _ = repl.history.read_from_file();
        }

        repl
    }

    /// Gives current command based on the cursor
    fn cur<'a>(&'a self, c: &Cursor, lines: &'a Vec<String>) -> &'a Vec<String> {
        if c.use_history {
            // unwrap because if use_history is enabled, there must be at least one element in
            // history
            self.history.cur().unwrap()
        } else {
            lines
        }
    }

    /// Easy access to the current line
    fn cur_str<'a>(&'a self, c: &Cursor, lines: &'a Vec<String>) -> &'a str {
        &self.cur(c, lines)[c.lineno]
    }

    /// Copy the lines from history into the lines buffer
    fn replace_with_history(&self, lines: &mut Vec<String>) {
        let cur = self.history.cur().unwrap();
        lines.resize(cur.len(), String::new());

        for (i, string) in cur.iter().enumerate() {
            lines[i].clear();
            lines[i] += string;
        }
    }

    fn exit(&mut self) -> ! {
        let _ = terminal::disable_raw_mode();
        println!();
        let _ = self.history.write_to_file();
        std::process::exit(0)
    }

    /// Print a command
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

        queue!(
            stdout,
            terminal::Clear(terminal::ClearType::CurrentLine),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        )?;
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
            )?;
            linter::print_linted(stdout, line)?;
            queue!(stdout, style::Print("\n"))?;
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

    /// The main function, gives the next command
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
                        self.exit()
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
                    event::KeyCode::Tab => {
                        if c.use_history {
                            self.replace_with_history(&mut lines);
                            c.use_history = false;
                        };

                        lines[c.lineno].insert_str(c.charno, "    ");
                        c.charno += 4;

                        &lines[c.lineno]
                    }

                    event::KeyCode::Home => {
                        c.charno = 0;
                        self.cur_str(&c, &lines)
                    }
                    event::KeyCode::End => {
                        let s = self.cur_str(&c, &lines);
                        c.charno = s.len();
                        s
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

                    event::KeyCode::PageUp => history_up!(retain self, stdout, c, lines, colour),
                    // At the top of the current block, go to previous history block
                    event::KeyCode::Up if c.lineno == 0 => {
                        history_up!(self, stdout, c, lines, colour)
                    }
                    // In the middle of a block, go up one line
                    event::KeyCode::Up => {
                        c.lineno -= 1;
                        queue!(stdout, cursor::MoveUp(1))?;
                        let s = self.cur_str(&c, &lines);
                        c.charno = min(s.len(), c.charno);
                        s
                    }

                    event::KeyCode::PageDown => {
                        history_down!(retain self, stdout, c, lines, colour)
                    }
                    // At the bottom of the block, and in history. This means that there are more
                    // blocks down, either further down the history or when history is over, the
                    // editable lines itself
                    event::KeyCode::Down
                        if c.use_history && (c.lineno + 1) == self.history.cur().unwrap().len() =>
                    {
                        history_down!(self, stdout, c, lines, colour)
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
                    event::KeyCode::Backspace if c.lineno > 1 => {
                        if c.use_history {
                            self.replace_with_history(&mut lines);
                            c.use_history = false;
                        };

                        c.lineno -= 1;
                        c.charno = lines[c.lineno].len();
                        let line = lines.remove(c.lineno + 1);
                        lines[c.lineno] += &line;

                        execute!(stdout, cursor::MoveUp(1))?;
                        self.print_lines(&mut stdout, &mut c, &lines, colour)?;

                        &lines[c.lineno]
                    }
                    event::KeyCode::Delete
                        if c.charno < self.cur_str(&c, &lines).chars().count() =>
                    {
                        if c.use_history {
                            self.replace_with_history(&mut lines);
                            c.use_history = false;
                        };

                        lines[c.lineno].remove(c.charno);
                        &lines[c.lineno]
                    }
                    event::KeyCode::Delete if (c.lineno + 1) < self.cur(&c, &lines).len() => {
                        if c.use_history {
                            self.replace_with_history(&mut lines);
                            c.use_history = false;
                        };

                        let line = lines.remove(c.lineno + 1);
                        lines[c.lineno] += &line;

                        self.print_lines(&mut stdout, &mut c, &lines, colour)?;

                        &lines[c.lineno]
                    }

                    event::KeyCode::Enter => {
                        let is_command = if !c.use_history && lines.len() == 1 {
                            match &lines[0] as &str {
                                "exit" => {
                                    self.exit();
                                }
                                ".tree" => {
                                    self.show_tree = !self.show_tree;
                                    queue!(
                                        stdout,
                                        style::Print("\n"),
                                        cursor::MoveToColumn(0),
                                        if self.show_tree {
                                            style::Print("Showing tree")
                                        } else {
                                            style::Print("Hiding tree")
                                        },
                                        style::Print("\n"),
                                        cursor::MoveToColumn(0),
                                    )?;
                                    true
                                }
                                "clear" => {
                                    c.charno = 0;
                                    lines[0].clear();

                                    queue!(
                                        stdout,
                                        terminal::Clear(terminal::ClearType::All),
                                        cursor::MoveTo(0, 0),
                                    )?;

                                    true
                                }
                                _ => false,
                            }
                        } else {
                            false
                        };

                        if is_command {
                            c.charno = 0;
                            lines[0].clear();
                            execute!(
                                stdout,
                                style::SetForegroundColor(colour),
                                style::Print(self.leader),
                                style::ResetColor,
                            )?;

                            continue;
                        }

                        if c.use_history && (c.lineno + 1) == self.history.cur().unwrap().len() {
                            break;
                        }
                        let indent = get_indent(&self.cur(&c, &lines)[0..(c.lineno + 1)]);

                        if !c.use_history && (c.lineno + 1) == lines.len() && indent == 0 {
                            break;
                        } else {
                            if c.use_history {
                                self.replace_with_history(&mut lines);
                                c.use_history = false;
                            }

                            c.lineno += 1;
                            c.charno = (indent * 4) as usize;
                            lines.insert(c.lineno, " ".repeat(c.charno));
                            execute!(stdout, style::Print("\n"))?;
                            self.print_lines(&mut stdout, &mut c, &lines, colour)?;
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

            queue!(stdout, style::Print(leader))?;
            linter::print_linted(&mut stdout, s)?;
            execute!(
                stdout,
                cursor::MoveToColumn((leader_len + c.charno + 1) as u16)
            )?;
        }

        terminal::disable_raw_mode()?;
        println!();

        let src = self.cur(&c, &lines).join("\n");

        if c.use_history {
            self.history.push(self.history.cur().unwrap().clone());
        } else {
            self.history.push(lines);
        }

        Ok(src)
    }
}

pub fn get_persistant_file_path() -> Option<PathBuf> {
    match app_dirs2::data_root(app_dirs2::AppDataType::UserCache) {
        Ok(mut p) => {
            p.push(".anilang-history");
            Some(p)
        }
        Err(_) => None,
    }
}

#[derive(Debug, Default)]
struct Cursor {
    use_history: bool,
    lineno: usize,
    charno: usize,
}
