

use super::*;

use crate::Ir::expr::*;

impl Parser {

    
    pub fn eval_expr(&mut self) -> Vec<RpnExpr> {
        let mut output: Vec<RpnExpr> = Vec::new();
        let mut op_stack: Vec<Token> = Vec::new();
        let mut previous_token: Option<Token> = None;
        while !matches!(
            self.peek(0).token,
            TokenType::Semi | TokenType::OpenScope | TokenType::Coma
        ) {
            if self.peek(0).token == TokenType::CloseParen && self.peek(1).token == TokenType::Semi {
                break;
            }
            let token = self.consume();
            let token_copy = token.clone();
            
            match token.token {
                TokenType::Num | TokenType::CharValue => {
                    output.push(RpnExpr::PushNum(PushNum { data: token }));
                }
                
                
                
                TokenType::Var => {
                    
                    if self.peek(0).token == TokenType::Dot {
                        self.consume();
                        let strcut_var = self.consume();
                        let res = GetStructValue {
                            var_name: token.value.unwrap(),
                            struct_value_name: strcut_var.value.unwrap(),
                        };
                        output.push(RpnExpr::GetStructValue(res));
                        continue;
                        
                    }
                    
                    if token.value.as_deref() == Some("sizeof") {
                        self.consume();
                        let var: Token = self.consume();
                        self.consume();
                        let res = GetSizeOf {
                            var,
                        };
                        output.push(RpnExpr::GetSizeOf(res));
                        continue;
                    }
                    
                    if self.peek(0).token == TokenType::OpenParen {
                        let func = self.parse_rpn_function(token);
                        output.push(func);
                    } else if self.peek(0).token == TokenType::OpenBracket {
                        self.consume();
                        //redo this to take expr
                        let index = self.consume();
                        self.consume();
                        let get_array_value = GetArrayValue {
                            name: token,
                            index,
                        };
                        output.push(RpnExpr::GetArrayValue(get_array_value));
                    } else {
                        output.push(RpnExpr::PushVar(PushVar { data: token }));
                    }
                }
                
                
                TokenType::Address => {
                    let var = self.consume();
                    let res = GetAddr {
                        var,
                    };  
                    output.push(RpnExpr::GetAddr(res));
                    continue;
                }
                
                
                TokenType::OpenParen => {
                    op_stack.push(token);
                }
                
                TokenType::CloseParen => {
                    while let Some(op) = op_stack.last() {
                        if op.token == TokenType::OpenParen {
                            break;
                        }
                        let op = op_stack.pop().unwrap();
                        output.push(RpnExpr::Operator(Operator { data: op }));
                    }
                    op_stack.pop(); // pop '('
                }
                
                _ if Parser::is_operator(&token) => {
                    if previous_token.is_some() {
                        
                        if Parser::is_operator(&previous_token.clone().unwrap()) && token.token == TokenType::Mul {
                            let mut stack_depth = 1;
                            while self.peek(0).token == TokenType::Mul {
                                stack_depth += 1;
                                self.consume();
                            }
                            let var = self.consume();
                            let res = Deref {
                                var,
                                stack_depth,
                            };
                            output.push(RpnExpr::Deref(res));
                            continue;
                        }
                        
                        if Parser::is_operator(&previous_token.clone().unwrap()) && token.token == TokenType::Sub {
                            output.push(RpnExpr::Negative(Negative { data: self.consume() }));
                            continue;
                        }
                    }
                    else {
                        if token.token == TokenType::Mul {
                            let mut stack_depth = 1;
                            while self.peek(0).token == TokenType::Mul {
                                stack_depth += 1;
                                self.consume();
                            }
                            let var = self.consume();
                            let res = Deref {
                                var,
                                stack_depth,
                            };
                            output.push(RpnExpr::Deref(res));
                            continue;
                        }
                        
                        if token.token == TokenType::Sub {
                            output.push(RpnExpr::Negative(Negative { data: self.consume() }));
                            continue;
                        }
                    }
                    while let Some(top) = op_stack.last() {
                        if top.token == TokenType::OpenParen {
                            break;
                        }
                        
                        if Parser::bigger_operator(&token)
                        <= Parser::bigger_operator(top)
                        {
                            let op = op_stack.pop().unwrap();
                            output.push(RpnExpr::Operator(Operator { data: op }));
                        } else {
                            break;
                        }
                    }
                    op_stack.push(token);
                }
                
                _ => {}
            }
            previous_token = Some(token_copy);
        }
        
        while let Some(op) = op_stack.pop() {
            output.push(RpnExpr::Operator(Operator { data: op }));
        }
        
        
        output
    }
}