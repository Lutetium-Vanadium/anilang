use super::Value;
use crossterm::style::Colorize;
use gc::Gc;
use std::cell::RefCell;
use std::collections::HashSet;

#[derive(Default)]
struct CyclicDetection {
    ids: HashSet<usize>,
    should_check: bool,
}

thread_local! {
    static CYCLE_DETECTION: RefCell<CyclicDetection> = RefCell::new(Default::default());
}

/// Wrapper around &Value to allow printing for cyclic values. std::fmt::{Debug, Display} do not
/// allow for passing a context while printing the values. This means, if there is a cyclic value,
/// it will keep following the cycle and printing until it crashes. This simple wrapper, creates a
/// context which keeps track of values encountered, and prints '[ cyclic ]' the second time it
/// encounters a value.
pub struct FmtValue<'a>(pub &'a Value);

impl fmt::Display for FmtValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        CYCLE_DETECTION.with(|ids| {
            let mut ids = ids.borrow_mut();
            ids.should_check = true;
            ids.ids.clear();
        });
        write!(f, "{}", self.0)?;
        CYCLE_DETECTION.with(|ids| {
            ids.borrow_mut().should_check = false;
        });
        Ok(())
    }
}

impl fmt::Debug for FmtValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        CYCLE_DETECTION.with(|ids| {
            let mut ids = ids.borrow_mut();
            ids.should_check = true;
            ids.ids.clear();
        });

        write!(f, "{}", self.0)?;

        CYCLE_DETECTION.with(|ids| {
            ids.borrow_mut().should_check = false;
        });
        Ok(())
    }
}

/// prints a value to stdout. This is equivalent to wrapping a Value in FmtValue and printing that
/// using `println!()`. See FmtValue for more.
#[inline]
pub fn print_value(value: &Value, dbg: bool) {
    if dbg {
        println!("{:?}", FmtValue(value));
    } else {
        println!("{}", FmtValue(value));
    }
}

#[inline]
fn write_cyclic(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", "[cyclic]".dark_grey())
}

fn detected_cycle(id: usize) -> bool {
    CYCLE_DETECTION.with(|cycle_detection| {
        let mut cycle_detection = cycle_detection.borrow_mut();

        if cycle_detection.should_check {
            if cycle_detection.ids.contains(&id) {
                return true;
            }

            cycle_detection.ids.insert(id);
        }

        false
    })
}

/// When printing we want to only show the inner value, which is what the user expects
/// for example for an integer 1, when printing, the user expects for it to be printed as
/// `1` and not Value::Int(1)
use std::fmt;
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(ref s) => write!(f, "{}", s.borrow()),
            Value::List(ref l) => {
                if detected_cycle(Gc::id(l)) {
                    return write_cyclic(f);
                }

                let l = l.borrow();
                // Arbitrary number after which it should be pretty printed in multiple lines
                if l.len() < 8 {
                    write!(f, "{:?}", l)
                } else {
                    write!(f, "{:#?}", l)
                }
            }
            Value::Object(ref o) => {
                if detected_cycle(Gc::id(o)) {
                    return write_cyclic(f);
                }

                let o = o.borrow();
                // Arbitrary number after which it should be pretty printed in multiple lines
                if o.len() < 3 {
                    write!(f, "{:?}", o)
                } else {
                    write!(f, "{:#?}", o)
                }
            }
            Value::Range(s, e) => write!(f, "{} -> {}", s, e),
            Value::Function(ref func) => write!(f, "{}", func),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(ref s) => {
                let s = &s.borrow();
                // while printing quotes must be escaped to avoid confusion
                if s.contains('\'') && !s.contains('"') {
                    write!(f, "\"{}\"", s)
                } else {
                    write!(f, "'")?;
                    for i in s.chars() {
                        if i == '\'' {
                            write!(f, "\\{}", i)?;
                        } else {
                            write!(f, "{}", i)?;
                        }
                    }
                    write!(f, "'")
                }
            }
            Value::List(ref l) => {
                if detected_cycle(Gc::id(l)) {
                    return write_cyclic(f);
                }

                let l = l.borrow();
                // Arbitrary number after which it should be pretty printed in multiple lines
                if l.len() < 8 {
                    write!(f, "{:?}", l)
                } else {
                    write!(f, "{:#?}", l)
                }
            }
            Value::Object(ref o) => {
                if detected_cycle(Gc::id(o)) {
                    return write_cyclic(f);
                }

                let o = o.borrow();
                // Arbitrary number after which it should be pretty printed in multiple lines
                if o.len() < 3 {
                    write!(f, "{:?}", o)
                } else {
                    write!(f, "{:#?}", o)
                }
            }
            Value::Range(s, e) => write!(f, "{}..{}", s, e),
            Value::Function(ref func) => write!(f, "{}", func),
            Value::Int(i) => write!(f, "{:?}", i),
            Value::Float(fl) => write!(f, "{:?}", fl),
            Value::Bool(b) => write!(f, "{:?}", b),
            Value::Null => write!(f, "null"),
        }
    }
}
