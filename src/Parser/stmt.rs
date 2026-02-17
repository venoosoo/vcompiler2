use super::*;

use crate::Ir::stmt::*;

use crate::Ir::expr::PushNum;


impl Parser {
    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        if Parser::is_type(self.peek(0)) {
            let type_token = self.consume(); 
            let var_token  = self.consume();

            // pointer
            if var_token.token == TokenType::Mul {
                let mut stack_depth: u32 = 1;
                while self.peek(0).token == TokenType::Mul {
                    stack_depth += 1;
                    self.consume();
                }
                let var_name = self.consume();
                if self.peek(0).token == TokenType::OpenParen {
                    return self.parse_func(var_name, (type_token, stack_depth));
                }

                if self.peek(0).token == TokenType::Eq {
                    let expr = self.eval_expr();
                    if self.peek(0).token != TokenType::Semi {
                        panic!("excpected semi colon");
                    }
                    self.consume();
                    let res = CreatePointer {
                        type_: type_token.token,
                        var: var_name.value.unwrap(),
                        stmt: expr,
                        pointer_depth: stack_depth,
                    };
                    return Some(Stmt::CreatePointer(res))

                } else if self.peek(0).token == TokenType::Semi {
                    self.consume();
                    let mut res: Vec<RpnExpr> = Vec::new();
                    let expr = RpnExpr::PushNum(PushNum { data: Token { token: TokenType::Num, value: Some("0xDEADBEEFDEADBEEF".to_string()) } });
                    res.push(expr);
                    let some =  CreatePointer { 
                        type_:TokenType::Num, 
                        var: var_name.value.unwrap(), 
                        stmt: res,
                        pointer_depth: stack_depth,
                    };
                    return Some(Stmt::CreatePointer(some));

                }
            }

            //init array 
            if self.peek(0).token == TokenType::OpenBracket {
                self.consume();
                let arr_size = self.consume();
                self.consume();
                let mut data: Vec<Token> = Vec::new();
                if self.peek(0).token == TokenType::Eq {
                    self.consume();
                    self.consume();
                    while self.peek(0).token != TokenType::CloseScope {
                        let num = self.consume();
                        if self.peek(0).token == TokenType::Coma {
                            self.consume();
                        }
                        data.push(num);

                    }
                    self.consume();
                }
                let init_array = InitArray {
                    name: var_token,
                    arr_type: type_token,
                    size: arr_size,
                    data: data,
                };

                if self.peek(0).token != TokenType::Semi {
                    panic!("Expected semi colon");
                }
                self.consume();

                return Some(Stmt::InitArray(init_array));
            }

            

            if self.peek(0).token == TokenType::Semi {
                self.consume();
                let some =  PushNum { data: Token { token: TokenType::Num, value: Some("0".to_string()) } };
                let expr = RpnExpr::PushNum(some);
                let mut res: Vec<RpnExpr> = Vec::new();
                res.push(expr);

                let new_var = CreateVar {
                    Type: type_token.token,
                    var: var_token.value.clone().unwrap(),
                    stmt: res,
                };
                
                return Some(Stmt::Var(new_var));
            }
            // create var
            if self.peek(0).token == TokenType::Eq {
                self.consume(); // Consume '='


                let res: Vec<RpnExpr> = self.eval_expr();
                
                if self.peek(0).token != TokenType::Semi {
                    panic!("Expected semi colon");
                }
                self.consume();
                let new_var = CreateVar {
                    Type: type_token.token,
                    var: var_token.value.clone().unwrap(),
                    stmt: res,
                };
                
                return Some(Stmt::Var(new_var));
            }
            // init function
            else if self.peek(0).token == TokenType::OpenParen {
                // the pointer depth will always be zero because if we had * in return type
                // it would be in another section
                return self.parse_func(var_token, (type_token, 0));

            }
        }

        if self.peek(0).token == TokenType::Mul {
            self.consume();
            let mut pointer_depth: u32 = 1;
            while self.peek(0).token == TokenType::Mul {
                self.consume();
                pointer_depth += 1;
            }
            let var = self.consume();
            if self.peek(0).token == TokenType::Eq {
                self.consume();
                let expr = self.eval_expr();
                if self.peek(0).token != TokenType::Semi {
                    panic!("no semi colon");
                }
                self.consume();
                let res = ChangePtrValue {
                    var: var.value.unwrap(),
                    stmt: expr,
                    pointer_depth,
                };
                return Some(Stmt::ChangePtrValue(res));

            } else {
                panic!("strange syntax pointer");
            }
        }

        if self.peek(0).token == TokenType::OpenScope {
            self.consume();
            let res = OpenScope { };
            return Some(Stmt::OpenScope(res));
        }
        if self.peek(0).token == TokenType::CloseScope {
            self.consume();
            let res = CloseScope {  };
            return Some(Stmt::CloseScope(res));
        }
        if self.peek(0).token == TokenType::Var {
            let var = self.consume();


            if self.peek(0).token == TokenType::Access {
                self.consume();
                let struct_var = self.consume();
                if self.peek(0).token == TokenType::Eq {
                    self.consume();
                    let expr = self.eval_expr();
                    if self.peek(0).token != TokenType::Semi {
                        panic!("excpected semi colon");
                    }
                    self.consume();
                    let res = ChangePtrStructValue {
                        struct_name: var.value.unwrap(),
                        value_name: struct_var.value.unwrap(),
                        expr,
                    };
                    return Some(Stmt::ChangePtrStructValue(res))
                }

            }


            if self.peek(0).token == TokenType::Dot {
                self.consume();
                let struct_var = self.consume();
                if self.peek(0).token == TokenType::Eq {
                    self.consume();
                    let expr = self.eval_expr();
                    if self.peek(0).token != TokenType::Semi {
                        panic!("excpected semi colon");
                    }
                    self.consume();
                    let res = ChangeStructValue {
                        struct_name: var.value.unwrap(),
                        value_name: struct_var.value.unwrap(),
                        expr,
                    };
                    return Some(Stmt::ChangeStructValue(res))
                }
            }


            if var.value.as_deref() == Some("asm") {
                let mut asm_code: Vec<String> = Vec::new();
                self.consume();
                while self.peek(0).token != TokenType::CloseScope {
                    let str = self.consume();
                    asm_code.push(str.value.unwrap());
                }
                self.consume();
                let res = AsmCode {
                    code: asm_code,
                };
                return Some(Stmt::AsmCode(res))
            }
            // change array element
            if self.peek(0).token == TokenType::OpenBracket {
                self.consume();
                let element = self.consume();
                self.consume();
                if self.peek(0).token == TokenType::Eq {
                    self.consume();
                    let res = self.eval_expr();
                    if self.peek(0).token != TokenType::Semi {
                        panic!("Expected semi colon");
                    }
                    self.consume();
                    let change_arr_elemnet = ChangeArrElement {
                        arr_name: var,
                        element,
                        expr: res,
                    };
                    return Some(Stmt::ChangeArrElement(change_arr_elemnet));
                }
            }


            if self.peek(0).token == TokenType::Eq {
                self.consume();
                let res = self.eval_expr();
                if self.peek(0).token != TokenType::Semi && self.peek(0).token != TokenType::CloseParen {
                    panic!("Expected semi colon or ')'");
                }
                self.consume();
                let change_var = ChangeVar {
                    stmt: res,
                    var: var.value.unwrap(),
                };
                return Some(Stmt::ChangeVar(change_var));
            }
            if self.peek(0).token == TokenType::Inc {
                self.consume();
                if self.peek(0).token != TokenType::CloseParen {
                    if self.peek(0).token != TokenType::Semi {
                        println!("excpected ;");
                    }
                    else {
                        self.consume();
                    }
                }
                let inc_var = IncVar {
                    var
                };
                return  Some(Stmt::IncVar(inc_var));
            }
            else if self.peek(0).token == TokenType::Dec {
                self.consume();
                if self.peek(0).token != TokenType::CloseParen {
                    if self.peek(0).token != TokenType::Semi {
                        println!("excpected ;");
                    }
                    else {
                        self.consume();
                    }
                }
                self.consume();
                let dec_var = DecVar {
                    var
                };
                return  Some(Stmt::DecVar(dec_var));
            }
            // function call
            else if self.peek(0).token == TokenType::OpenParen {
                let func_call = self.gen_init_func(var);
                self.consume();
                return Some(Stmt::FunctionCall(func_call));
                
            }
        }
        if self.peek(0).token == TokenType::If {
            self.consume();
            let res = self.eval_expr();
            let mut expr_arr: Vec<Stmt> = Vec::new(); 
            while self.peek(0).token != TokenType::CloseScope && self.peek(0).token != TokenType::Else {
                let expr = self.parse_stmt().unwrap();
                expr_arr.push(expr);
            }
            if self.peek(0).token != TokenType::Else {
                    let expr = self.parse_stmt().unwrap();
                    expr_arr.push(expr);
                }
            let mut else_expr_arr: Vec<Stmt> = Vec::new(); 
            if self.m_tokens.len() >= 1{
                if self.peek(0).token == TokenType::Else {
                    self.consume();
                    while self.peek(0).token != TokenType::CloseScope {
                        let expr = self.parse_stmt().unwrap();
                        else_expr_arr.push(expr);
                    }
                    let expr = self.parse_stmt().unwrap();
                    else_expr_arr.push(expr);
                    
                }
            }
            let if_var = IfStmt {
                expr: res,
                data: expr_arr,
                else_data: else_expr_arr,
            };
            return Some(Stmt::IfStmt(if_var));
        }
        if self.peek(0).token == TokenType::While {
            self.consume();
            let res = self.eval_expr();
            let mut expr_arr: Vec<Stmt> = Vec::new();
            while self.peek(0).token != TokenType::CloseScope {
                let expr = self.parse_stmt().unwrap();
                expr_arr.push(expr);
            }
            let while_var = WhileStmt {
                expr: res,
                data: expr_arr,
            };
            return Some(Stmt::WhileStmt(while_var));
        }
        if self.peek(0).token == TokenType::For {
            self.consume();
            if self.peek(0).token != TokenType::OpenParen {
                panic!("excpected '('");
            }
            self.consume();
            let first_expr = Box::new(self.parse_stmt().unwrap());
            let second_expr = self.eval_expr();
            self.consume();
            let third_expr = Box::new(self.parse_stmt().unwrap());
            if self.peek(0).token != TokenType::CloseParen {
                panic!("excpected ')'");
            }
            self.consume();
            let mut expr_arr: Vec<Stmt> = Vec::new(); 
            while self.peek(0).token != TokenType::CloseScope {
                let expr = self.parse_stmt().unwrap();
                expr_arr.push(expr);
            }
            let for_var = ForStmt {
                expr1: first_expr,
                expr2: second_expr,
                expr3: third_expr,
                data: expr_arr,
            };
            return Some(Stmt::ForStmt(for_var));
        }

        if self.peek(0).token == TokenType::Struct {
            self.consume();
            let struct_name = self.consume();
            if self.peek(0).token == TokenType::OpenScope {
                //init of struct
                self.consume();
                let mut counter = 0;
                let mut elements: HashMap<String, StructArg> = HashMap::new();
                while self.peek(0).token != TokenType::CloseScope {
                    let arg_type = self.consume();
                    let mut pointer_depth = 0;
                    if self.peek(0).token == TokenType::Mul {
                        pointer_depth += 1;
                        self.consume();
                        while self.peek(0).token == TokenType::Mul {
                            pointer_depth += 1;
                            self.consume();
                        }
                    }

                    let name = self.consume();
                    if self.peek(0).token != TokenType::Semi {
                        panic!("expceted semi colon");
                    }
                    self.consume();
                    let res = StructArg {
                        name: name.clone(),
                        arg_type: arg_type,
                        pointer_depth,
                        pos: counter,
                    };
                    counter += 1;
                    elements.insert(name.value.unwrap(), res);
                }
                self.consume(); // CloseScope
                let res = InitStruct {
                    name: struct_name.value.unwrap(),
                    elements,
                };
                if self.peek(0).token != TokenType::Semi {
                    panic!("excpected semi colon");
                }
                self.consume();
                return Some(Stmt::InitStruct(res));
            }
            else {
                let mut pointer_depth = 0;
                let mut expr: Option<Vec<RpnExpr>> = None;
                while self.peek(0).token == TokenType::Mul {
                    self.consume();
                    pointer_depth += 1;
                }
                let var_name = self.consume();
                if self.peek(0).token == TokenType::Eq {
                    self.consume();
                    if self.peek(0).token == TokenType::OpenScope {
                        todo!()
                    }
                    else {
                        expr = Some(self.eval_expr());
                    }

                }
                let res = CreateStruct {
                    var_name: var_name.value.unwrap(),
                    struct_name: struct_name.value.unwrap(),
                    pointer_depth,
                    expr,
                };
                if self.peek(0).token != TokenType::Semi {
                    panic!("nigga place that semi colon");
                }
                self.consume();
                
                return Some(Stmt::CreateStruct(res))
                
            }
        }

        if self.peek(0).token == TokenType::Return {
            self.consume();
            let expr = self.eval_expr();
            if self.peek(0).token != TokenType::Semi {
                panic!("excpected ;");
            }
            self.consume();
            let return_ = Ret {
                expr: expr,
                func_name: self.func_name.clone(),
            };
            return Some(Stmt::Ret(return_));
        }
        None
    }
}