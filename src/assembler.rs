use regex::Regex;
use std::collections::HashMap;
use std::fs::{read_to_string, File};

pub fn parse_filename(configs: &[String]) -> Result<&String, &'static str> {
    if configs.len() == 0 {
        return Err("missing filename argument");
    }
    Ok(&configs[1])
}

pub fn read_lines(filename: &str) -> Vec<String> {
    read_to_string(filename)
        .unwrap()
        .lines()
        .map(String::from)
        .map(|line| line.trim().to_string())
        .collect()
}

pub struct Parser {
    contents: Vec<String>,
    currentLine: usize,
    currentInstruction: String,
}

impl Parser {
    pub fn new(contents: Vec<String>) -> Self {
        return Parser {
            contents: contents,
            currentLine: 0,
            currentInstruction: "".to_string(),
        };
    }

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

    fn instructionType(&self) -> Option<&str> {
        match self.contents[self.currentLine].as_str() {
            line if line.starts_with('@') => Some("A_INSTRUCTION"),
            line if line.starts_with('(') && line.ends_with(')') => Some("L_INSTRUCTION"),
            line if !line.is_empty() => Some("C_INSTRUCTION"),
            _ => None,
        }
    }

    fn symbol(&self) -> Option<String> {
        match self.instructionType() {
            Some("A_INSTRUCTION") => Some(self.currentInstruction[1..].to_string()),
            Some("L_INSTRUCTION") => {
                Some(self.currentInstruction[1..self.currentInstruction.len() - 1].to_string())
            }
            Some("C_INSTRUCTION") => None,
            _ => None,
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

pub struct Code {}

impl Code {
    pub fn new() -> Self {
        return Code {};
    }

    fn dest(&self, mnemonic: &str) -> String {
        match mnemonic {
            "M" => "001".to_string(),
            "D" => "010".to_string(),
            "DM" | "MD" => "011".to_string(),
            "A" => "100".to_string(),
            "AM" | "MA" => "101".to_string(),
            "AD" | "DA" => "110".to_string(),
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

#[derive(Debug)]
pub struct SymbolTable {
    symbols: HashMap<String, usize>,
    mem_counter: usize, // starts at 16
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
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

    fn addEntry(&mut self, key: &str, val: &usize) {
        self.symbols.insert(key.to_string(), val.clone());
    }

    fn contains(&self, key: &str) -> bool {
        self.symbols.contains_key(key)
    }

    fn getAddress(&self, key: &str) -> Option<&usize> {
        match self.symbols.get(key) {
            Some(value) => return Some(value),
            None => None,
        }
    }
}

pub fn assemble_hack_code(mut parser: Parser, mut symbols: SymbolTable, code: Code) -> Vec<String> {
    // symbols table first pass
    // builds the labels keys
    // increment line_counter for A and C instructions only

    let mut label_counter = 0;

    while parser.hasMoreLines() {
        parser.advance();
        dbg!("{:?}", &parser.currentInstruction);
        dbg!("{:?}", &parser.instructionType());
        match parser.instructionType() {
            Some("A_INSTRUCTION") => {
                label_counter += 1;
            }
            Some("C_INSTRUCTION") => {
                label_counter += 1;
            }
            Some("L_INSTRUCTION") => {
                symbols.addEntry(&parser.symbol().expect("L instruction"), &label_counter);
            }
            _ => {}
        }
    }

    dbg!("{:}", &symbols);

    fn handle_address_instruction(
        symbol: &String,
        symbol_table: &mut SymbolTable,
    ) -> Option<String> {
        if symbol.parse::<i32>().is_ok() {
            return Some(format! {"{:016b}", symbol.parse::<i32>().unwrap()});
        } else {
            match symbol_table.getAddress(&symbol) {
                Some(value) => return Some(format! {"{:016b}", value}),
                None => {
                    // get current variable mem counter
                    let variable_counter = symbol_table.mem_counter;
                    // increment variable mem counter
                    symbol_table.mem_counter += 1;
                    symbol_table.addEntry(&symbol, &variable_counter);
                    return Some(format! {"{:016b}", variable_counter});
                }
            }
        }
    }

    // second pass, adds address to variables if not a label (start from mem[16]...)
    parser.currentLine = 0;
    let mut compiled_code: Vec<String> = Vec::new();
    while parser.hasMoreLines() {
        parser.advance();
        dbg!("{:?}", &parser.currentInstruction);
        match parser.instructionType() {
            Some("A_INSTRUCTION") => {
                let a_code =
                    handle_address_instruction(&parser.symbol().expect("error"), &mut symbols)
                        .expect("could not find symbol...");
                dbg!("{:?}, {:?}", &a_code, &parser.symbol().unwrap());
                compiled_code.push(a_code);
            }
            Some("C_INSTRUCTION") => {
                compiled_code.push(format!(
                    "111{}{}{}",
                    code.comp(&parser.comp()),
                    code.dest(&parser.dest()),
                    code.jump(&parser.jump())
                ));
            }
            Some("L_INSTRUCTION") => {}
            _ => {}
        }
    }
    println!("{:?}", compiled_code);
    compiled_code
}
