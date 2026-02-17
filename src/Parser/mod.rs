
use std::collections::HashMap;

use crate::Ir::expr::RpnExpr;
use crate::Tokenizer::{Token, TokenType};

use crate::Ir::stmt::Stmt;

pub mod expr;
pub mod stmt;
pub mod function;


pub struct Parser {
    m_tokens: Vec<Token>,
    m_index: usize,
    expressions: Vec<Stmt>,
    func_name: String,
}



pub struct Program(Vec<Stmt>);

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, expr) in self.0.iter().enumerate() {
            writeln!(f, "[{}] {:?}", i, expr)?; 
        }
        Ok(())
    }
}



impl Parser {
    pub fn new(m_tokens: Vec<Token>) -> Self {
        Parser {
            m_tokens,
            m_index: 0,
            expressions: Vec::new(),
            func_name: String::new(),
        }
    }

    fn peek(&self, offset: usize) -> &Token {
        let pos: usize = self.m_index + offset;
        if self.m_index >= self.m_tokens.len() {
            panic!("Trying to parse token more than token array has\nm_index: {}",self.m_tokens.len());
        }
        &self.m_tokens[pos]
    }



    

    fn consume(&mut self) -> Token {
        if self.m_index >= self.m_tokens.len() {
            panic!("Trying to consume more than m_src len");
        }
        self.m_tokens.remove(0)
    }



    fn bigger_operator(token: &Token) -> i32 {
        match token.token {
            TokenType::Add => 1,
            TokenType::Sub => 1,
            TokenType::Mul => 2,
            TokenType::Div => 2,
            TokenType::And => 1,
            TokenType::Or => 1,
            TokenType::AsertEq => 1,
            TokenType::Not => 1,
            TokenType::NotEq => 1,
            TokenType::Less => 1,
            TokenType::LessThan => 1,
            TokenType::More => 1,
            TokenType::Remainder => 2,
            TokenType::MoreThan => 1,
            _ => panic!("Starnge token in bigger_operator"),
        }
    }

    fn is_operator(token: &Token) -> bool {
        match  token.token {
            TokenType::Add => true,
            TokenType::Sub => true,
            TokenType::Mul => true,
            TokenType::Div => true,
            TokenType::And => true,
            TokenType::Or => true,
            TokenType::AsertEq => true,
            TokenType::Not => true,
            TokenType::NotEq => true,
            TokenType::Less => true,
            TokenType::LessThan => true,
            TokenType::More => true,
            TokenType::Remainder => true,
            TokenType::MoreThan => true,
            _ => false,
        }
    }

    fn is_type(token: &Token) -> bool {
        match token.token {
            TokenType::IntType => true,
            TokenType::CharType => true,
            TokenType::LongType => true,
            TokenType::ShortType => true,
            TokenType::Void => true,
            _ => false,
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        while !self.m_tokens.is_empty() {
            if let Some(stmt) = self.parse_stmt() {
                self.expressions.push(stmt);
            } else {
                println!("tokens: {:?}",self.m_tokens);
                panic!("Unexpected token: {:?} at {}", self.peek(0).token, self.m_index);
            }
        }

        self.expressions.clone()
    }


}




