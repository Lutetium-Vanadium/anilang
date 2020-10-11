#[macro_export]
macro_rules! history_up {
    ($self:ident, $stdout:ident, $c:ident, $lines:ident, $colour:ident) => {{
        $c.use_history = true;

        let lines = match $self.history.prev() {
            Some(s) => {
                $self.print_lines(&mut $stdout, &mut $c, &s, $colour)?;
                $c.lineno = s.len() - 1;
                queue!($stdout, cursor::MoveDown($c.lineno as u16))?;
                s
            }
            None => match $self.history.cur() {
                Some(s) => s,
                None => {
                    $c.use_history = false;
                    &$lines
                }
            },
        };

        let s = &lines[$c.lineno];
        let s_len = s.chars().count();

        if $c.charno == 0 || $c.charno > s_len {
            $c.charno = s_len;
        }
        s
    }};
}

#[macro_export]
macro_rules! history_down {
    ($self:ident, $stdout:ident, $c:ident, $lines:ident, $colour:ident) => {{
        let lines = match $self.history.next() {
            Some(s) => s,
            None => {
                $c.use_history = false;
                &$lines
            }
        };

        queue!($stdout, cursor::MoveUp($c.lineno as u16))?;
        $c.lineno = 0;
        $self.print_lines(&mut $stdout, &mut $c, lines, $colour)?;

        let s = &lines[$c.lineno];
        let s_len = s.chars().count();

        if $c.charno == 0 || $c.charno > s_len {
            $c.charno = s.chars().count();
        }
        s
    }};
}
