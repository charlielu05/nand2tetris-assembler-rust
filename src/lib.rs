pub mod assembler;

#[cfg(test)]
mod tests {
    use crate::assembler::{parse_filename, read_lines, Code, Parser, SymbolTable};
    use std::vec;

    #[test]
    fn test_parse_filename() {
        let test_configs = ["foo".to_string(), "bar".to_string()];
        assert!(parse_filename(&test_configs).unwrap() == "bar");
    }

    #[test]
    fn test_read_lines() {
        let filename = "./test_files/test.asm";
        let code_lines = read_lines(filename);
        assert!(code_lines.len() == 10);
        // check first code line
        assert!(code_lines[0] == "// Compute R1=1+...+R0");
        // check last code line
        assert!(code_lines[code_lines.len() - 1] == "AM=D|M");
    }

    #[test]
    fn test_parser() {
        let test_string = vec![
            "// comment".to_string(),
            "@10".to_string(),
            "(LABEL)".to_string(),
            "dest=cmp;jmp".to_string(),
        ];
        let mut parser = Parser::new(test_string);
        // test more lines
        assert_eq!(parser.hasMoreLines(), true);
        parser.advance();
        // test that comment is skipped and first instruction is an A instruction
        assert!(parser.instructionType() == Some("A_INSTRUCTION"));
        parser.advance();
        // test that tht next instruction is L instruction
        assert!(parser.instructionType() == Some("L_INSTRUCTION"));
        parser.advance();
        assert!(parser.instructionType() == Some("C_INSTRUCTION"));
        // assert no more lines
        assert_eq!(parser.hasMoreLines(), false);
    }
}
