use clap::{Parser};
use std::{fs::File, io::{Read, Write}};

mod tokenizer;
mod parser;
mod r#gen;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
   #[arg(short, long, required = true, help = "provide file main.v")]
   file: String,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = Cli::parse();

    println!("file name is: {}", cli.file);

    let mut file = File::open(cli.file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    println!("file contains: {}",&contents);

    let mut tokenizer = tokenizer::Tokenizer::new(contents);
    tokenizer.tokenize();
    println!("{}",tokenizer);

    let mut parser = parser::Parser::new(tokenizer.m_res);
    let res = parser.parse();
    
    // to lazy to make normal debug print
    println!("parse result\n{:#?}",res);

    let mut generator = r#gen::Gen::new(res);
    let asm = generator.gen_asm()?;
    let mut file = File::create("main.asm")?;
    file.write(asm.as_bytes())?;


    Ok(())
}