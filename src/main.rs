use crossterm::style::Colorize;
use crossterm::ErrorKind as CEK;
use std::io::ErrorKind as IEK;
use std::path::PathBuf;
use structopt::StructOpt;

mod compiler;
mod repl;
mod runtime;
mod syntax;

#[derive(Debug, StructOpt)]
/// Anilang is a programming language written for fun in rust. Details about it can be found at
/// `https://github.com/Lutetium-Vanadium/anilang#readme`
struct Opt {
    /// The file to compile. If `BIN FILE` not given, the binary will be written to a file with same
    /// name as FILE but no extension.
    #[structopt(
        name = "SRC FILE",
        short = "c",
        long = "compile",
        parse(from_os_str),
        conflicts_with("interpret_file")
    )]
    compile_file: Option<PathBuf>,

    /// The file to interpret.
    #[structopt(
        name = "FILE",
        short = "i",
        long = "interpret",
        parse(from_os_str),
        conflicts_with("compile_file")
    )]
    interpret_file: Option<PathBuf>,

    /// The file name for the binary to be written to or executed based on the whether the compile
    /// argument is given.
    #[structopt(
        name = "BIN FILE",
        parse(from_os_str),
        conflicts_with("interpret_file")
    )]
    bin_file: Option<PathBuf>,

    /// To print a quick guide through the syntax
    #[structopt(long, short)]
    syntax: bool,
}

fn main() {
    let opt = Opt::from_args();

    if opt.syntax {
        syntax::print();
    } else if let Some(input_file) = opt.compile_file {
        let output_file = opt.bin_file.unwrap_or_else(|| {
            let mut output_file = input_file.clone();
            output_file.set_extension("");
            output_file
        });

        compiler::compile(input_file, output_file).unwrap_or_else(|e| {
            print!("{} ", "ERROR".dark_red());
            match e {
                CEK::Utf8Error(_) => {
                    println!("Found non utf-8 bytes while trying to process a string.");
                    println!(
                        "Make sure the file you are trying to compile is a utf-8 encoded text file"
                    );
                }
                CEK::IoError(e) => {
                    println!(
                        "An error occurred while compiling the file\nError message: {}",
                        e
                    )
                }
                e => println!(
                    "An unknown error occurred, could not compile the program.\nError message: {}",
                    e
                ),
            }
        });
    } else if let Some(bin_file) = opt.bin_file {
        // .into() doesn't work to convert io::Result to crossterm::Result
        runtime::run(bin_file).unwrap_or_else(|e| {
            print!("{} ", "ERROR".dark_red());
            match e.kind() {
                IEK::InvalidData => {
                    println!(
                        "Unable to process the file, make sure to supply the correct compiled file"
                    );
                }
                _ => {}
            }
            println!("Error message: {}", e);
        });
    } else if let Some(file) = opt.interpret_file {
        runtime::interpret(file).unwrap_or_else(|e| {
            print!("{} ", "ERROR".dark_red());
            match e {
                CEK::Utf8Error(_) => {
                    println!("Found non utf-8 bytes while trying to process a string.");
                    println!(
                        "Make sure the file you are trying to compile is a utf-8 encoded text file"
                    );
                }
                CEK::IoError(e) => {
                    println!(
                        "An error occurred while compiling the file\nError message: {}",
                        e
                    )
                }
                e => println!(
                    "An unknown error occurred, could not compile the program.\nError message: {}",
                    e
                ),
            }
        });
    } else {
        repl::run();
    }
}
