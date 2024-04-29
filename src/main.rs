use regex::Regex;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs::{read_to_string, File};
use std::io::{self, Write};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::rc::Rc;

fn parse_filename(configs: &[String]) -> Result<&String, &'static str> {
    if configs.len() == 0 {
        return Err("missing filename argument");
    }
    Ok(&configs[1])
}

fn read_lines(filename: &str) -> Vec<String> {
    read_to_string(filename)
        .unwrap()
        .lines()
        .map(String::from)
        .map(|line| line.trim().to_string())
        .collect()
}

struct Parser {
    contents: Vec<String>,
    currentLine: usize,
    currentInstruction: String,
}

impl Parser {
    fn hasMoreLines(&self) -> bool {
        if self.currentLine < self.contents.len() - 1 {
            return true;
        } else {
            return false;
        }
    }

    fn advance(&mut self) {
        if self.hasMoreLines() {
            self.currentLine += 1;
            if self.contents[self.currentLine].starts_with("//") {
                self.advance();
            } else {
                self.currentInstruction = self.contents[self.currentLine].to_string();
            }
        }
    }

    fn instructionType(&self) -> String {
        match self.contents[self.currentLine].as_str() {
            line if line.starts_with('@') => "A_INSTRUCTION".to_string(),
            line if line.starts_with('(') && line.ends_with(')') => "L_INSTRUCTION".to_string(),
            _ => "C_INSTRUCTION".to_string(),
        }
    }

    fn symbol(&self) -> String {
        match self.instructionType().as_str() {
            "A_INSTRUCTION" => self.currentInstruction[1..].to_string(),
            "L_INSTRUCTION" => {
                self.currentInstruction[1..self.currentInstruction.len() - 1].to_string()
            }
            "C_INSTRUCTION" => "".to_string(),
            _ => "".to_string(),
        }
    }

    fn dest(&self) -> String {
        let re = Regex::new(r"^(?P<dest>[^=]*?)\s*=").unwrap();
        // dbg!(re.captures(&self.currentInstruction));
        match re.captures(&self.currentInstruction) {
            Some(caps) => caps["dest"].to_string(),
            None => "".to_string(),
        }
    }

    fn comp(&self) -> String {
        let re = Regex::new(
            r"(?P<dest>[AMD]{1,3}=)?(?P<comp>[01\-AMD!|+&><]{1,3})(?P<jump>;[JGTEQELNMP]{3})??",
        )
        .unwrap();
        // dbg!(re.captures(&self.currentInstruction));
        match re.captures(&self.currentInstruction) {
            Some(caps) => caps["comp"].to_string(),
            None => "".to_string(),
        }
    }

    fn jump(&self) -> String {
        let re = Regex::new(r"(?:.*;\s*)(?P<jump>\S*)").unwrap();
        // dbg!(re.captures(&self.currentInstruction));
        match re.captures(&self.currentInstruction) {
            Some(caps) => caps["jump"].to_string(),
            None => "".to_string(),
        }
    }
}

struct Code {}

impl Code {
    fn dest(&self, mnemonic: &str) -> String {
        match mnemonic {
            "M" => "001".to_string(),
            "D" => "010".to_string(),
            "DM" => "011".to_string(),
            "A" => "100".to_string(),
            "AM" => "101".to_string(),
            "AD" => "110".to_string(),
            "ADM" => "111".to_string(),
            _ => "000".to_string(),
        }
    }

    fn jump(&self, mnemonic: &str) -> String {
        match mnemonic {
            "JGT" => "001".to_string(),
            "JEQ" => "010".to_string(),
            "JGE" => "011".to_string(),
            "JLT" => "100".to_string(),
            "JNE" => "101".to_string(),
            "JLE" => "110".to_string(),
            "JMP" => "111".to_string(),
            _ => "000".to_string(),
        }
    }

    fn comp(&self, mnemonic: &str) -> String {
        match mnemonic {
            "0" => "0101010".to_string(),
            "1" => "0111111".to_string(),
            "-1" => "0111010".to_string(),
            "D" => "0001100".to_string(),
            "A" => "0110000".to_string(),
            "!D" => "0001101".to_string(),
            "!A" => "0110001".to_string(),
            "-D" => "0001111".to_string(),
            "-A" => "0110011".to_string(),
            "D+1" => "0011111".to_string(),
            "A+1" => "0110111".to_string(),
            "D-1" => "0001110".to_string(),
            "A-1" => "0110010".to_string(),
            "D+A" => "0000010".to_string(),
            "D-A" => "0010011".to_string(),
            "A-D" => "0000111".to_string(),
            "D&A" => "0000000".to_string(),
            "D|A" => "0010101".to_string(),
            "M" => "1110000".to_string(),
            "!M" => "1110001".to_string(),
            "-M" => "1110011".to_string(),
            "M+1" => "1110111".to_string(),
            "M-1" => "1110010".to_string(),
            "D+M" => "1000010".to_string(),
            "D-M" => "1010011".to_string(),
            "M-D" => "1000111".to_string(),
            "D&M" => "1000000".to_string(),
            "D|M" => "1010101".to_string(),
            _ => "".to_string(),
        }
    }
}

struct SymbolTable {
    symbols: HashMap<String, u32>,
    mem_counter: u32, // starts at 16
}

impl SymbolTable {
    fn new() -> SymbolTable {
        SymbolTable {
            symbols: HashMap::from([
                ("R0".to_string(), 0),
                ("R1".to_string(), 1),
                ("R2".to_string(), 2),
                ("R3".to_string(), 3),
                ("R4".to_string(), 4),
                ("R5".to_string(), 5),
                ("R6".to_string(), 6),
                ("R7".to_string(), 7),
                ("R8".to_string(), 8),
                ("R9".to_string(), 9),
                ("R10".to_string(), 10),
                ("R11".to_string(), 11),
                ("R12".to_string(), 12),
                ("R13".to_string(), 13),
                ("R14".to_string(), 14),
                ("R15".to_string(), 15),
                ("SP".to_string(), 0),
                ("LCL".to_string(), 1),
                ("ARG".to_string(), 2),
                ("THIS".to_string(), 3),
                ("THAT".to_string(), 4),
                ("SCREEN".to_string(), 16384),
                ("KBD".to_string(), 24576),
            ]),
            mem_counter: 16,
        }
    }

    fn addEntry(&mut self, key: &str, val: &u32) {
        self.symbols.insert(key.to_string(), val.clone());
        self.mem_counter += 1;
    }

    fn contains(&self, key: &str) -> bool {
        self.symbols.contains_key(key)
    }

    fn getAddress(&self, key: &str) -> &u32 {
        match self.symbols.get(key) {
            Some(value) => return value,
            None => return &1,
        }
    }
}

fn assemble_hack_code(mut parser: Parser, mut symbols: SymbolTable, code: Code) -> Vec<String> {
    // symbols table first pass

    let mut line_counter = 0;

    while parser.hasMoreLines() {
        parser.advance();
        match parser.instructionType().as_str() {
            "A_INSTRUCTION" => {}
            "C_INSTRUCTION" => {}
            "L_INSTRUCTION" => {
                symbols.addEntry(&parser.symbol(), &(line_counter));
            }
            _ => {}
        }
        line_counter += 1;
    }
    dbg!(symbols.contains("LOOP"));
    dbg!(symbols.getAddress("LOOP"));
    // symbols table second pass
    parser.currentLine = 0;
    symbols.mem_counter = 16;
    println!("{:?}", &mut symbols.symbols);

    fn handle_second_pass(symbol: &String, symbol_table: &mut SymbolTable) {
        if !symbol_table.contains(symbol) {
            symbol_table.addEntry(symbol, &symbol_table.mem_counter.clone());
        }
    }

    while parser.hasMoreLines() {
        parser.advance();
        match parser.instructionType().as_str() {
            "A_INSTRUCTION" => handle_second_pass(&parser.symbol(), &mut symbols),
            _ => {}
        }
    }

    fn handle_address_instruction(symbol: &String, symbol_table: &SymbolTable) -> Option<String> {
        if !symbol_table.contains(&symbol) {
            return None;
        } else {
            return Some(format! {"{:016b}", symbol_table.getAddress(&symbol)});
        }
    }

    parser.currentLine = 0;
    let mut compiled_code: Vec<String> = Vec::new();
    while parser.hasMoreLines() {
        parser.advance();
        println!("{:?}", &parser.currentLine);
        match parser.instructionType().as_str() {
            "A_INSTRUCTION" => compiled_code.push(
                handle_address_instruction(&parser.symbol(), &symbols)
                    .expect("could not find symbol..."),
            ),
            "C_INSTRUCTION" => {
                compiled_code.push(format!(
                    "111{}{}{}",
                    code.comp(&parser.comp()),
                    code.dest(&parser.dest()),
                    code.jump(&parser.jump())
                ));
                dbg!(&parser.comp());
                dbg!(code.comp(&parser.comp()));
                dbg!(&parser.dest());
                dbg!(code.dest(&parser.dest()));
                dbg!(&parser.jump());
                dbg!(code.jump(&parser.jump()));
            }
            "L_INSTRUCTION" => {}
            _ => {}
        }
    }
    println!("{:?}", compiled_code);
    compiled_code
}
fn main() {
    let args: Vec<String> = env::args().collect();

    let filename = parse_filename(&args).unwrap_or_else(|err| {
        println!("{}", err);
        std::process::exit(1);
    });

    println!("Assembling file: {}", filename);

    let lines = read_lines(filename);

    let parser = Parser {
        contents: lines,
        currentLine: 0,
        currentInstruction: "".to_string(),
    };
    let code = Code {};
    let symbols = SymbolTable::new();

    let assembly_code = assemble_hack_code(parser, symbols, code);

    // write to file
    let mut f = File::create("test.hack").expect("failed...");
    f.write(assembly_code.join("\n").as_bytes())
        .expect("failed...");
}
