use crate::types::Type;
use crate::value::{ErrorKind, Result, Value};
use std::cell::RefCell;
use std::io::{self, prelude::*};
use std::rc::Rc;

pub type NativeFn = fn(Vec<Value>) -> Result<Value>;

pub fn print(args: Vec<Value>) -> Result<Value> {
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

pub fn input(args: Vec<Value>) -> Result<Value> {
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

pub fn push(mut args: Vec<Value>) -> Result<Value> {
    if args.len() != 2 {
        return Err(ErrorKind::IncorrectArgCount {
            expected: 2,
            got: args.len(),
        });
    }

    let to_push = args.pop().unwrap();

    match &args[0] {
        Value::List(l) => {
            l.borrow_mut().push(to_push);
            Ok(Value::Null)
        }
        _ => Err(ErrorKind::IncorrectType {
            got: args[0].type_(),
            expected: Type::List.into(),
        }),
    }
}

pub fn pop(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(ErrorKind::IncorrectArgCount {
            expected: 1,
            got: args.len(),
        });
    }

    match &args[0] {
        Value::List(l) => {
            let mut l = l.borrow_mut();
            if l.len() == 0 {
                return Err(ErrorKind::Other {
                    message: "Cannot pop from empty list".to_owned(),
                });
            }

            Ok(l.pop().unwrap())
        }
        _ => Err(ErrorKind::IncorrectType {
            got: args[0].type_(),
            expected: Type::List.into(),
        }),
    }
}

pub fn assert(args: Vec<Value>) -> Result<Value> {
    for arg in args {
        if !bool::from(&arg) {
            return Err(ErrorKind::Other {
                message: format!("Assertion failed: {} is not truthy", arg),
            });
        }
    }

    Ok(Value::Null)
}
