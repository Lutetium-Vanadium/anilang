use crate::value::{ErrorKind, Result, Value};
use std::cell::RefCell;
use std::io::{self, prelude::*};
use std::rc::Rc;

pub type NativeFn = for<'a> fn(&'a [Value]) -> Result<Value>;

pub(crate) fn print(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        println!();
        return Ok(Value::Null);
    }

    for value in &args[..(args.len() - 1)] {
        print!("{} ", value)
    }

    println!("{}", args.last().unwrap());

    Ok(Value::Null)
}

pub(crate) fn input(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(ErrorKind::IncorrectArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    if let Some(arg) = args.last() {
        print!("{} ", arg);
    }

    io::stdout().flush().unwrap();
    let mut s = String::new();
    io::stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");

    // Remove the ending new line
    let new_len = s.trim_end_matches(|c| c == '\n' || c == '\r').len();
    s.truncate(new_len);

    Ok(Value::String(Rc::new(RefCell::new(s))))
}
