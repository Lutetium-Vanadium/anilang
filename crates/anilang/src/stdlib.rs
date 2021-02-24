use anilang::function::{native, Function};
use anilang::{Scope, Value};
use std::rc::Rc;

macro_rules! declare_native_fn {
    ($scope:expr => $fn_name:ident) => {
        $scope
            .declare(
                stringify!($fn_name).into(),
                Value::Function(Rc::new(Function::native_fn(native::$fn_name))),
            )
            .unwrap_or_else(|_| {
                panic!("Could not declare native function {}", stringify!($fn_name))
            })
    };
}

pub fn make_std() -> Rc<Scope> {
    let scope = Rc::new(Scope::new(0, None));

    declare_native_fn!(scope => print);
    declare_native_fn!(scope => input);
    declare_native_fn!(scope => assert);

    scope
}
