mod scanner;
mod common;
mod token;
mod keywords;
mod lexer;
mod parser;
mod symtab;
mod symbol;
mod plat;
mod gen_ir;
mod intercode;

use failure;
use crate::gen_ir::GenIR;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::symtab::SymTab;

pub fn run() -> Result<(), failure::Error> {
    let mut scanner = Scanner::new("./test_file/compiler.txt".to_string())?;
    let mut lexer = Lexer::new(&mut scanner);
    let mut sym_tab = Box::new(SymTab::new());
    let gen_ir = Some(Box::new(GenIR::new(sym_tab.clone())));
    let mut parser = Parser::new(&mut lexer, &mut sym_tab, gen_ir);
    parser.analyze();

    Ok(())
}