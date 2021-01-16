use crate::repl::AnilangLangInterface;
use crossterm::style::Colorize;
use shelp::LangInterface;

fn print_block(stdout: &mut std::io::Stdout, string: &str) {
    println!("{}", "  |".dark_blue());
    for (i, line) in string.lines().enumerate() {
        print!("{} ", format!("{} |", i).dark_blue());
        let _ = AnilangLangInterface::print_line(stdout, line);
        println!();
    }
    println!("{}", "  |".dark_blue());
}

pub fn print() {
    let stdout = &mut std::io::stdout();
    println!("The current syntax is subject to change.\n");
    println!("To declare a variable:");
    print_block(stdout, "let a = <val>");
    println!("\nOnce declared variables can be reassigned to any other value");
    print_block(stdout, "a = <val>");
    println!();

    println!(
        r##"Basic arithmetic and boolean operators exist:
┌──────────┬────────────────────────────────────────────┐
│ operator │                  purpose                   │
├──────────┼────────────────────────────────────────────┤
│     +    │ Addition and String and List concatenation │
│     -    │ Subtraction                                │
│     *    │ Multiplication                             │
│     /    │ Division                                   │
│     %    │ Modulo                                     │
│     ^    │ Power                                      │
│    ||    │ Boolean Or                                 │
│    &&    │ Boolean And                                │
│    ==    │ Equality                                   │
│    !=    │ Not equal                                  │
│     >    │ Greater than                               │
│    >=    │ Greater than equal to                      │
│     <    │ Less than                                  │
│    <=    │ Less than equal to                         │
└──────────┴────────────────────────────────────────────┘

There are also conditionals"##,
    );
    print_block(
        stdout,
        r##"if <cond> {
    ...
} else if <cond> {
    ...
} else {
    ...
}"##,
    );

    println!("\nCurrently there are 2 kinds of loops");
    print_block(
        stdout,
        r##"loop {
    ...
}

while <cond> {
    ...
}"##,
    );
    let _ = AnilangLangInterface::print_line(stdout, "loop");
    println!(" provides an infinite loop");

    println!("\nFunctions can be declared in the following ways:");
    print_block(
        stdout,
        r##"// Regular function declaration, gets stored in <func_name>
fn <func_name>(<args>...) {
    ...
}

// Anonymous function which is immediately invoked, but it can be used
// like any other value
(fn(a, b) { a + b })(1, 2)"##,
    );

    print!("\nFunctions by default return the value of the last statement, but early");
    print!(" returns are possible with the ");
    let _ = AnilangLangInterface::print_line(stdout, "return");
    println!(" keyword");
    print_block(
        stdout,
        r##"fn factorial(n) {
    if n == 2 {
        return n
    }

    n * factorial(n-1)
}"##,
    );

    println!("\nStrings can be indexed using []");
    print_block(stdout, "\"string\"[1]\nvariable[2]");

    println!("\nThey can also be assigned to");
    print_block(
        stdout,
        r##"let variable = "----"
variable[2] = "a"  // variable is "--a-"
variable[1] = "ab" // variable is "-aba-""##,
    );

    print!("\nStrings larger than 1 character will remove the character at that index");
    println!("and insert the characters given");

    println!("\nThere are also comments -");
    print!("{} Single line: ", "-".grey());
    let _ = AnilangLangInterface::print_line(stdout, "// comment");
    print!("\n{} Multi line:  ", "-".dark_grey());
    let _ = AnilangLangInterface::print_line(stdout, "/* comment */");
    println!();
}
