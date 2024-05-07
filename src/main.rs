use hack_assembler::assembler::{
    assemble_hack_code, parse_filename, read_lines, Code, Parser, SymbolTable,
};
use std::env;
use std::fs::{read_to_string, File};
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();

    let filename = parse_filename(&args).unwrap_or_else(|err| {
        println!("{}", err);
        std::process::exit(1);
    });

    println!("Assembling file: {}", filename);

    let lines = read_lines(filename);
    let parser = Parser::new(lines);
    let code = Code::new();
    let symbols = SymbolTable::new();

    let assembly_code = assemble_hack_code(parser, symbols, code);

    // write to file
    let mut f = File::create("test.hack").expect("failed...");
    f.write(assembly_code.join("\n").as_bytes())
        .expect("failed...");
}
