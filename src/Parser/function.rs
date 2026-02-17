

use super::*;
use crate::Ir::stmt::{Arg, FunctionCall, InitFunc};

use crate::Ir::expr::Function;

impl Parser {

    pub fn parse_rpn_function(&mut self, name: Token) -> RpnExpr {
        // consume '('
        self.consume();

        let mut args = Vec::new();

        while self.peek(0).token != TokenType::CloseParen {
            let tok = self.consume();

            if tok.token != TokenType::Coma {
                args.push(tok);
            }
        }

        // consume ')'
        self.consume();

        RpnExpr::Function(Function {
            name,
            args,
        })
    }


    pub fn gen_init_func(&mut self,var: Token) -> FunctionCall {
        if self.peek(0).token != TokenType::OpenParen {
            panic!("Expected '('");
        }
        self.consume();
        let mut args: Vec<Vec<RpnExpr>> = Vec::new();
        while self.peek(0).token != TokenType::CloseParen {
            let expr = self.eval_expr();
            if self.peek(0).token == TokenType::Coma {
                self.consume();
            }
            args.push(expr);
        }
        if self.peek(0).token != TokenType::CloseParen {
            panic!("Expected ')'");
        }
        self.consume();
        let func_call = FunctionCall {
            name: var,
            args,
        };
        func_call
    }
    pub fn parse_func(&mut self, var_token: Token, type_token: (Token, u32)) -> Option<Stmt> {
        self.consume();
        let mut args: Vec<Arg> = Vec::new();
        let mut pointer_depth = 0;
        while self.peek(0).token != TokenType::CloseParen {
            let arg_type = self.consume();
            let mut struct_arg_name: Option<String> = None;

            if arg_type.token == TokenType::Struct {
                let struct_name = self.consume();
                struct_arg_name = Some(struct_name.value.unwrap());
            }
            
            while self.peek(0).token == TokenType::Mul {
                pointer_depth += 1;
                self.consume();
            }
            let arg_name = self.consume();
            if self.peek(0).token == TokenType::Coma {
                self.consume();
            }
            let arg = Arg {
                struct_name: struct_arg_name,
                arg_type,
                pointer_depth,
                name: arg_name,
            };
            args.push(arg);
        
        }
        self.consume();
        let mut expr_arr: Vec<Stmt> = Vec::new();
        let mut depth = 0;
        self.func_name = var_token.value.clone().unwrap();
        while self.peek(0).token != TokenType::CloseScope || depth > 0 {
            let expr: Stmt = self.parse_stmt().unwrap();
            match &expr {
                Stmt::ForStmt(_) => depth += 1,
                Stmt::WhileStmt(_) => depth += 1,
                Stmt::CloseScope(_) => depth -=1,
                _ => depth += 0,
            }
            expr_arr.push(expr);
        }
        let init_func = InitFunc {
            name: var_token,
            return_type: type_token,
            args,
            data: expr_arr
        };

        return Some(Stmt::InitFunc(init_func));
    }
}