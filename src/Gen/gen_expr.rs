
use crate::Ir::expr::RpnExpr;

use super::*;
use crate::Ir::expr::*;
use crate::Tokenizer::{Token, TokenType};

impl Gen {
    pub fn eval_expr(&mut self, rpn: Vec<RpnExpr>) {
        let mut stack: Vec<ExprStack> = Vec::new();
        for expr in rpn {
            match expr {
                RpnExpr::PushNum(p) => {
                    let val = p.data.value.clone().unwrap();
                    
                    if stack.is_empty() {
                        self.emit(format!("    mov rax, {}",val));
                        stack.push(ExprStack { reg: "rax".to_string(), var_typd: (TokenType::Num, 0) })
                    } else if stack.len() == 1 {
                        self.emit(format!("    mov rbx, {}", val));
                        stack.push(ExprStack { reg: "rbx".to_string(), var_typd: (TokenType::Num, 0) })
                    } else {
                        let slot = Gen::calc_expr_stack_size(&stack) + 8;
                        self.emit(format!(
                            "    mov QWORD [rbp-{}], {}",
                            slot, val
                        ));
                        stack.push(ExprStack { reg: format!("[rbp-{}]", slot), var_typd: (TokenType::Num, 0) })
                    }
                }

                RpnExpr::GetStructValue(v) => {
                    let var_data = self.m_vars.get(&v.var_name).expect(&format!("no var with name: {}",v.var_name));
                    let stack_pos = var_data.stack_pos;
                    if let Some(val) = var_data.struct_data.as_ref() {
                        let struct_data = self.structs.get(&val.struct_name).expect(&format!("no struct with name: {:?}",val.struct_name));
                        let element_size = struct_data.element_size;
                        let res = struct_data.elements.get(&v.struct_value_name).expect(&format!("in var: {:?} there's no field: {:?}",v.var_name, v.struct_value_name));
                                let value = stack_pos - (res.pos as i32 * element_size as i32);
                                if stack.is_empty() {
                                    self.emit(format!("    mov {} {}, [rbp - {}]",Gen::get_word(res.arg_type.token),Gen::get_rax_register(res.arg_type.token), value));
                                    stack.push(ExprStack { reg: "rax".to_string(), var_typd: (TokenType::Num, 0) })
                                } else if stack.len() == 1 {
                                    self.emit(format!("    mov {} {}, [rbp - {}]",Gen::get_word(res.arg_type.token), Gen::get_rbx_register(res.arg_type.token) ,value));
                                    stack.push(ExprStack { reg: "rbx".to_string(), var_typd: (TokenType::Num, 0) })
                                } else {
                                    let slot = Gen::calc_expr_stack_size(&stack) + 8;
                                    self.emit(format!(
                                        "    mov QWORD [rbp-{}], [rbp - {}]",
                                        slot, value
                                    ));
                                    stack.push(ExprStack { reg: format!("[rbp-{}]", slot), var_typd: (TokenType::Num, 0) })
                                }
                            }
                        
                }

                RpnExpr::GetSizeOf(v) => {
                    let name = v.var.value.unwrap();
                    let var_data = self.m_vars.get(&name).expect(&format!("no var with name: {}",name));
                    let mut size = self.get_size(var_data.var_type);
                    if var_data.pointer_depth > 0 {
                        size = 8;
                    }
                    if stack.is_empty() {
                        self.emit(format!("    mov eax, {}",size));
                        stack.push(ExprStack { reg: "rax".to_string(), var_typd: (TokenType::Num, 0) })
                    } else if stack.len() == 1 {
                        self.emit(format!("    mov rbx, {}",size));
                        stack.push(ExprStack { reg: "rbx".to_string(), var_typd: (TokenType::Num, 0) })
                    } else {
                        let slot = Gen::calc_expr_stack_size(&stack) + 8;
                        self.emit(format!(
                            "    mov QWORD [rbp-{}], {}",
                            slot, size
                        ));
                        stack.push(ExprStack { reg: format!("[rbp-{}]", slot), var_typd: (TokenType::Num, 0) })
                    }
                }

                RpnExpr::Deref(v) => {
                    let name = v.var.value.unwrap();
                    let var_data = self.m_vars.get(&name).expect(&format!("no var with name: {}",name));
                    let pointer_depth = var_data.pointer_depth;
                    let var_type = var_data.var_type;
                    self.emit(format!("    mov rsi, [rbp - {}]",var_data.stack_pos));
                    for i in 0..v.stack_depth {
                        if i % 2 == 0 {
                            self.emit(format!("    mov rax, [rsi]"));
                        } else {
                            self.emit(format!("    mov rsi, [rax]"));
                        }
                    }
                    if v.stack_depth % 2 == 0 {
                        self.emit(format!("    mov rax, rsi"));
                    }
                    stack.push(ExprStack { reg: "rax".to_string(), var_typd: (var_type, pointer_depth) })

                }


                RpnExpr::GetAddr(v) => {
                    let var_name = v.var.value.unwrap();
                    let var_data = self.m_vars.get(&var_name).expect(&format!("no var with name: {}",var_name));
                    let pointer_depth = var_data.pointer_depth;
                    let var_type = var_data.var_type;
                    self.emit(format!("    lea rsi, [rbp-{}]",var_data.stack_pos));
                    if stack.is_empty() {
                        self.emit(format!("    mov rax, rsi"));
                        stack.push(ExprStack { reg: "rax".to_string(), var_typd: (var_type, pointer_depth + 1) })
                    } else if stack.len() == 1 {
                        self.emit(format!("    mov rbx, rsi"));
                        stack.push(ExprStack { reg: "rbx".to_string(), var_typd: (var_type, pointer_depth + 1) })
                    } else {
                        let slot = Gen::calc_expr_stack_size(&stack) + 8;
                        self.emit(format!(
                            "    mov QWORD [rbp-{}], rsi",
                            slot
                        ));
                        stack.push(ExprStack { reg: format!("[rbp-{}]", slot), var_typd: (var_type, pointer_depth + 1) })
                    }
                }

                RpnExpr::GetArrayValue(v) => {
                    let mut res: Vec<RpnExpr> = Vec::new();
                    
                    let push_var = PushVar {
                        data: v.name,
                    };
                    res.push(RpnExpr::PushVar(push_var));
                    if v.index.token != TokenType::Num {
                        let im_dumd = PushVar {
                            data: v.index
                        };
                        res.push(RpnExpr::PushVar(im_dumd));                                      
                    }
                    else {
                        let push_index = PushNum {
                            
                            data: v.index,
                        };
                        res.push(RpnExpr::PushNum(push_index));
                    }

                    let plus_op = Operator {
                        data: Token { token: TokenType::Add, value: Some("+".into()) }
                    };

                    res.push(RpnExpr::Operator(plus_op));

                    self.eval_expr(res);


                    self.emit(format!("    mov rax, [rax]"));
                    
                
                
                }

                RpnExpr::Negative(mut v) => {
                    if v.data.token == TokenType::Var {
                        let mut mask_vector: Vec<RpnExpr> = Vec::new();
                        let name = v.data.value.clone().unwrap();
                        let var = self.m_vars.get(&name).expect(format!("unkown var: {}",&name).as_str());
                        let arg = Gen::get_rax_register(var.var_type);
                        let push_var = PushVar {
                            data: v.data,
                        };
                        mask_vector.push(RpnExpr::PushVar(push_var));
                        self.eval_expr(mask_vector);
                        self.emit(format!("    neg {}",arg));
                    }
                    else if v.data.token == TokenType::Num {
                        let mut mask_vector: Vec<RpnExpr> = Vec::new();
                        v.data.value = Some(format!("-{}",v.data.value.unwrap()));
                        let push_num = PushNum {
                            data: v.data
                        };
                        mask_vector.push(RpnExpr::PushNum(push_num));
                        self.eval_expr(mask_vector);
                    }
                }

                
                RpnExpr::PushVar( p) => {
                    let name = p.data.value.clone().unwrap();
                    let var: &VarData = self.m_vars.get(&name).expect(format!("unkown var: {}",&name).as_str());
                    let pos = var.stack_pos;
                    let var_type = var.var_type;
                    let pointer_depth = var.pointer_depth;
                    
                    if stack.is_empty() {
                        let mut arg = Gen::get_rax_register(var_type);
                        if var.pointer_depth != 0 {
                            arg = "rax".to_string();
                        }
                        if var.arr_data.is_some() {
                            self.emit(format!("    lea rax, [rbp-{}]",pos));
                            stack.push(ExprStack { reg: format!("{}",arg), var_typd: (var_type,pointer_depth) });
                        }
                        else {
                            self.emit(format!("    mov {}, [rbp-{}]",arg ,pos));
                            stack.push(ExprStack { reg: format!("{}",arg), var_typd: (TokenType::Num,0) });
                        }
                    } else if stack.len() == 1 {
                        let mut arg = Gen::get_rbx_register(var_type);
                        if var.pointer_depth != 0 {
                            arg = "rbx".to_string();
                        }
                        if var.arr_data.is_some() {
                            self.emit(format!("    lea rbx, [rbp-{}]",pos));
                        }
                        else {
                            self.emit(format!("    mov {}, [rbp-{}]",arg ,pos));
                        }
                        stack.push(ExprStack { reg: format!("{}",arg), var_typd: (TokenType::Num,0) });
                    } else {
                        let slot = Gen::calc_expr_stack_size(&stack) + 8;
                        if var.arr_data.is_some() {
                            self.emit(format!("    lea [rbp - {}], [rbp - {}]",slot,pos));
                        }
                        else {
                            self.emit(format!("    mov [rbp - {}], [rbp-{}]",slot ,pos));
                        }
                        stack.push(ExprStack { reg: format!("[rbp-{}]", slot), var_typd: (TokenType::Num,0) });
                    }
                }
                
                RpnExpr::Operator(op) => {
                    let t = &op.data.token;
                    
                    match t {
                        // ===== binary ops =====
                        TokenType::Add
                        | TokenType::Sub
                        | TokenType::Mul
                        | TokenType::Div
                        | TokenType::Remainder => {
                            let rhs = stack.pop().expect("rhs missing");
                            let lhs = stack.pop().expect("lhs missing");
                            let res = self.compare_reg(&lhs.reg, &rhs.reg);

                            if *t == TokenType::Add {
                                if lhs.var_typd.1 > 0 {
                                    self.emit(format!("    imul {}, {}",res.1, self.get_size(lhs.var_typd.0)));
                                }
                                else if rhs.var_typd.1 > 0 {
                                    self.emit(format!("    imul {}, {}",res.0, self.get_size(rhs.var_typd.0)));
                                }
                            }

                            if *t == TokenType::Sub {
                                if lhs.var_typd.1 > 0 {
                                    self.emit(format!("    imul {}, {}",res.1, self.get_size(lhs.var_typd.0)));
                                }
                            }
                            
                            match t {
                                TokenType::Add => self.emit(format!("    add {}, {}", res.0, res.1)),
                                TokenType::Sub => self.emit(format!("    sub {}, {}", res.0, res.1)),
                                TokenType::Mul => self.emit(format!("    imul {}, {}", res.0, res.1)),
                                TokenType::Div => {
                                    self.emit("    cdq".into());
                                    self.emit(format!("    idiv {}", res.1));
                                }
                                TokenType::Remainder => {
                                    self.emit(format!("    cqo"));
                                    self.emit(format!("    idiv {}",res.1));
                                    self.emit(format!("    mov {}, rdx",res.0));
                                }
                                _ => unreachable!(),
                            }
                            stack.push(ExprStack { reg: format!("{}",lhs.reg), var_typd: (TokenType::Num,0) });
                        }
                        
                        // ===== comparisons =====
                        TokenType::AsertEq
                        | TokenType::NotEq
                        | TokenType::Less
                        | TokenType::LessThan
                        | TokenType::More
                        | TokenType::MoreThan => {
                            let rhs = stack.pop().unwrap();
                            let lhs = stack.pop().unwrap();
                            
                            let res = self.compare_reg(&lhs.reg, &rhs.reg);
                            
                            self.emit(format!("    cmp {}, {}",res.0, res.1));
                            
                            let set = match t {
                                TokenType::AsertEq  => "sete",
                                TokenType::NotEq    => "setne",
                                TokenType::Less     => "setl",
                                TokenType::LessThan => "setle",
                                TokenType::More     => "setg",
                                TokenType::MoreThan => "setge",
                                _ => unreachable!(),
                            };
                            
                            self.emit(format!("    {} al", set));
                            self.emit("    movzx rax, al".into());
                            
                            stack.push(ExprStack { reg: "rax".into(), var_typd: (TokenType::Num,0) });
                        }
                        
                        _ => self::panic!("unsupported operator {:?}", t),
                    }
                }
                
                RpnExpr::Function(func) => {
                    // let function_call = FunctionCall {
                    //     name: func.name,
                    //     args: func.args,
                    // };
                    // let expr = Expr::FunctionCall(function_call);
                    // self.parse_one_expr(expr);
                    
                    // stack.push("rax".into());
                }
            }
        }
    }
}