
use super::*;

use crate::Ir::Stmt;

impl Gen {
    pub fn parse_stmt(&mut self,expr: Stmt) {
        match expr {
                Stmt::Var(v) => {
                    let expr_type = self.get_type_of_expr(v.stmt.clone());
                    if expr_type.1 != 0 {
                        self::panic!("trying to create var with pointer value");
                    }
                    self.eval_expr(v.stmt);
                    let pos: i32 = self.alloc(v.Type) as i32;
                    self.emit(format!("    mov {} [rbp - {}], {}",Gen::get_word(v.Type),pos, Gen::get_rax_register(v.Type)));
                    self.add_var(v.var, VarData { 
                        stack_pos: pos, 
                        scope_depth: self.depth_size, 
                        var_type: v.Type, 
                        arr_data: None,
                        pointer_depth: expr_type.1,
                        struct_data: None
                     
                    });
                }
                Stmt::OpenScope(v) => {
                    self.scope_stack.push(self.m_stack_pos as i32);
                    self.depth_size += 1
                }
                Stmt::CloseScope(v) => {
                    self.m_stack_pos = self.scope_stack.pop().expect("unexcpected }") as u32;
                    self.m_vars.retain(|_, value| {
                        value.scope_depth != self.depth_size
                    });
                    self.depth_size -= 1;
                    
                }

                Stmt::CreatePointer(v) => {
                    let expr_type = self.get_type_of_expr(v.stmt.clone());
                    self.eval_expr(v.stmt);
                
                
                    if v.pointer_depth != expr_type.1 && expr_type.0 != TokenType::IntType {
                        println!("name: {}",v.var);
                        println!("lhs: {}, rhs: {}",v.pointer_depth,expr_type.1);
                        self::panic!("trying to create pointer var with unexcepted value");
                    }

                    if v.type_ != expr_type.0 {
                        self::panic!("trying to create pointer with wrong type value");
                    }
                    

                    // pointers takes 8 bytes no matter the real type
                    let pos: i32 = self.alloc(TokenType::LongType) as i32;
                    self.emit(format!("    mov [rbp - {}], rax",pos));
                    let var_data = VarData {
                        stack_pos: pos,
                        scope_depth: self.depth_size,
                        var_type: v.type_,
                        arr_data: None,
                        pointer_depth: v.pointer_depth,
                        struct_data: None

                    };
                    self.m_vars.insert(v.var, var_data);
                    
                }

                Stmt::ChangePtrValue(v) => {
                    let expr_type = self.get_type_of_expr(v.stmt.clone());
                    self.eval_expr(v.stmt);
                    let var_data = self.m_vars.get(&v.var).expect(&format!("no var with name: {}",v.var));
                    let var_type = var_data.var_type;
                    if var_type != expr_type.0 && expr_type.1 != v.pointer_depth {
                        self::panic!("when changing var: {}, unexcpected value",v.var)
                    }
                    self.emit(format!("    mov rsi, [rbp - {}]",var_data.stack_pos));
                    self.emit(format!("    mov {} [rsi], {}",Gen::get_word(var_type), Gen::get_rax_register(var_type)));
                }

                Stmt::InitArray(v) => {
                    let arr_size: u32 =v.size.value.unwrap().parse().unwrap();
                    let type_size = self.get_size(v.arr_type.token); 
                    let alloc_size = type_size * arr_size;
                    self.m_stack_pos += alloc_size;
                    let stack_pos: u32 = self.m_stack_pos.try_into().unwrap();
                    let mut amount_taken: u32 = 0;
                    if v.data.len() > arr_size as usize {
                        self::panic!("trying to init array with more numbers than size");
                    }
                    for i in v.data {
                        self.emit(format!("    mov {} [rbp - {}], {}",Gen::get_word(v.arr_type.token),(stack_pos - amount_taken * type_size), i.value.unwrap()));
                        amount_taken += 1;
                    }
                    let arr_data = ArrData {
                        size: arr_size,
                    };
                    let arr_var = VarData {
                        stack_pos: stack_pos as i32,
                        scope_depth: self.depth_size,
                        var_type: v.arr_type.token,
                        arr_data: Some(arr_data),
                        pointer_depth: 1,
                        struct_data: None
                    };
                    self.add_var(v.name.value.unwrap(), arr_var);

                }

                Stmt::ChangeVar(v) => {
                    self.eval_expr(v.stmt);
                    let var = self.m_vars.get(&v.var).unwrap();
                    self.emit(format!("    mov {} [rbp - {}], {}",Gen::get_word(var.var_type),var.stack_pos, Gen::get_rax_register(var.var_type)));   
                }
                
                Stmt::IfStmt(v) => {
                    self.eval_expr(v.expr);
                    self.emit("    cmp rax, 0".to_string());
                    let id = self.get_id();
                    if v.else_data.len() >= 1 {
                        self.emit(format!("    je else_{}",id));
                    }
                    else {
                        self.emit(format!("    je end_if_{}",id));
                    }
                    self.emit(format!("if_{}:",id));
                    for i in v.data {
                        self.parse_stmt(i);
                    }
                    if v.else_data.len() >= 1 {
                        self.emit(format!("    je end_if_{}",id));
                        self.emit(format!("else_{}:",id));
                        for i in v.else_data {
                            self.parse_stmt(i);
                        }
                        
                    }
                    self.emit(format!("end_if_{}:",id));
                    
                }



                Stmt::WhileStmt(v) => {
                    let id = self.get_id();
                    self.emit(format!("while_{}:",id));
                    self.eval_expr(v.expr);
                    self.emit("    cmp rax, 1".to_string());
                    self.emit(format!("    jne end_while_{}",id));
                    for i in v.data {
                        self.parse_stmt(i);
                    }
                    self.emit(format!("    jmp while_{}",id));
                    self.emit(format!("end_while_{}:",id));
                }
                Stmt::ForStmt(v) => {
                    let id = self.get_id();
                    self.depth_size += 1;
                    self.scope_stack.push(self.m_stack_pos as i32);
                    self.parse_stmt(*v.expr1.clone());
                    self.depth_size -= 1;
                    self.emit(format!("for_{}:",id));
                    self.eval_expr(v.expr2);
                    self.emit("    test rax, rax".to_string());
                    self.emit(format!("    je end_for_{}",id));
                    for i in v.data {
                        self.parse_stmt(i);
                    }
                    self.parse_stmt(*v.expr3);
                    self.emit(format!("    jmp for_{}",id));
                    self.emit(format!("end_for_{}:",id));
                    self.m_stack_pos = self.scope_stack.pop().expect("unexcpected }") as u32;

                }
                Stmt::IncVar(v) => {
                    let var = self.m_vars.get(&v.var.value.unwrap()).unwrap();
                    let pos = var.stack_pos;
                    let rax_reg = Gen::get_rax_register(var.var_type);
                    self.emit(format!("    mov {} {}, [rbp - {}]",Gen::get_word(var.var_type),rax_reg,pos));
                    self.emit(format!("    inc {}",rax_reg));
                    self.emit(format!("    mov [rbp - {}], {}",pos,rax_reg));
                }
                Stmt::DecVar(v) => {
                    let var = self.m_vars.get(&v.var.value.unwrap()).unwrap();
                    let pos = var.stack_pos;
                    self.emit(format!("    mov eax, [rbp - {}]",pos));
                    self.emit("    dec eax".to_string());
                    self.emit(format!("    mov [rbp - {}], eax",pos));
                }
                Stmt::Ret(v) => {
                    let type_expr = self.get_type_of_expr(v.expr.clone());
                    self.eval_expr(v.expr);
                    let func_data = self.functions.get(&v.func_name).expect(&format!("something wrong using return on unkown function: {}",v.func_name));
                    if type_expr.0 == func_data.return_type.0.token && type_expr.1 == func_data.return_type.1  {
                        self.emit("    mov rsp, rbp".to_string());
                        self.emit("    pop rbp".to_string());
                        self.emit("    ret".to_string());
                    }
                    else {
                        println!("type_expr: {:?}\nfunc_data: {:?}",type_expr,func_data);
                        self::panic!("trying to return with wrong type in: {}",v.func_name);
                    }
                }
                Stmt::InitStruct(v) => {
                    // we already added it earlier while checking for function init
                    // so we just skipping this to not make a copy
                }


                Stmt::ChangeStructValue(v) => {
                    let expr_type = self.get_type_of_expr(v.expr.clone());
                    let var_data = self.m_vars.get(&v.struct_name).expect(&format!("no var with name: {}",v.struct_name));
                    if let Some(val) = var_data.struct_data.as_ref() {
                        let struct_data = self.structs.get(&val.struct_name).expect(&format!("no struct with name: {:?}",v.struct_name));
                        let field = struct_data.elements.get(&v.value_name).expect(&format!("in var: {:?} there's no field: {:?}",v.struct_name, v.value_name));
                        if expr_type.0 == field.arg_type.token && expr_type.1 == field.pointer_depth {
                            let stack_pos = var_data.stack_pos as u32 - (field.pos * struct_data.element_size);
                            self.eval_expr(v.expr);
                            self.emit(format!("    mov {} [rbp - {}], {}",Gen::get_word(expr_type.0),stack_pos, Gen::get_rax_register(expr_type.0)));
                        }
                        else {
                            self::panic!("trying to assign var: {} in struct {} with wrong type",v.struct_name,v.value_name)
                        }
                        
                    }
                }
                   

                Stmt::CreateStruct(v) => {
                    if let Some(expr) = v.expr {
                        let expr_type = self.get_type_of_expr(expr.clone());
                        if expr_type.0 != TokenType::Struct {
                            self::panic!("wrong type when trying to assing to struct")
                        }
                        if v.pointer_depth != expr_type.1 {
                            self::panic!("wrong pointer depth when assing to struct")
                        }
                        self.eval_expr(expr);
                        let pos = self.alloc(TokenType::LongType);
                        self.emit(format!("    mov QWORD [rbp - {}], rax",pos));
                    }
                    if v.pointer_depth == 0 {
                        let struct_data = self.structs.get(&v.struct_name).expect(&format!("no struct with name: {:?}",v.struct_name));
                        self.m_stack_pos += struct_data.element_size * struct_data.elements.len() as u32;
                    }
                    let res = VarData {
                        stack_pos: self.m_stack_pos as i32,
                        scope_depth: self.depth_size,
                        var_type: TokenType::Struct,
                        arr_data: None,
                        pointer_depth: v.pointer_depth,
                        struct_data: Some(VarStructData {struct_name: v.struct_name}),
                    };
                    self.m_vars.insert(v.var_name, res);


                }

                Stmt::ChangePtrStructValue(v) => {
                    let expr_type = self.get_type_of_expr(v.expr.clone());
                    let var_data = self.m_vars.get(&v.struct_name).expect(&format!("no var with name: {}",v.struct_name));
                    println!("m_vars: {:?}",self.m_vars);
                    if let Some(val) = var_data.struct_data.as_ref() {
                        let struct_pos = var_data.stack_pos;
                        println!("struct_pos: {}",struct_pos);
                        let struct_data = self.structs.get(&val.struct_name).expect(&format!("no struct with name: {:?}",v.struct_name));
                        let element_size = struct_data.element_size;
                        let field = struct_data.elements.get(&v.value_name).expect(&format!("in var: {:?} there's no field: {:?}",v.struct_name, v.value_name));
                        let field_pos = field.pos;
                        if expr_type.0 == field.arg_type.token && expr_type.1 == field.pointer_depth {
                            self.emit(format!("    mov rsi, [rbp - {}]",struct_pos));
                            self.emit(format!("    add rsi, {}",field_pos * element_size));
                            self.eval_expr(v.expr);
                            self.emit(format!("    mov [rsi], rax"));

                        }
                        else {
                            self::panic!("trying to assign var: {} in struct {} with wrong type",v.struct_name,v.value_name)
                        }
                        
                    }
                }

                Stmt::InitFunc(v) => {
                    let name = v.name.value.clone().unwrap();
                    self.current_func = name.clone();
                    self.emit(format!("{}:",name));
                    let mut temp_stack_size = self.calc_stack_size(&v.data);
                    for arg in &v.args {
                        if arg.pointer_depth > 0 {
                            temp_stack_size += 8;
                            continue;
                        }
                        else if let Some(val) = &arg.struct_name {
                            let struct_data = self.structs.get(val).expect(&format!("no struct with name: {}",val));
                            let size = struct_data.elements.len() as u32 * struct_data.element_size;
                            temp_stack_size += size;
                            continue;
                        
                        } 
                        else {
                            temp_stack_size += self.get_size(arg.arg_type.token);
                        }
                    }

                    
                    let total = (temp_stack_size + 15) & !15;
                    self.emit("    push rbp".to_string());
                    self.emit("    mov rbp, rsp".to_string());
                    self.emit(format!("    sub rsp, {}",total));
                    // so arg var will be local to the func
                    self.depth_size += 1;
                    self.scope_stack.push(self.m_stack_pos as i32);
                    for (index, arg) in v.args.iter().enumerate() {
                        if arg.pointer_depth > 0 {
                            let pos = self.alloc(TokenType::LongType);
                            self.emit(format!("    mov [rbp - {}], {}",pos, Gen::arg_pos(index,TokenType::LongType)));
                            let var_data = VarData { 
                                stack_pos: pos as i32, 
                                scope_depth: self.depth_size, 
                                var_type: arg.arg_type.token, 
                                arr_data:None, 
                                pointer_depth: arg.pointer_depth,
                                struct_data: {
                                    if let Some(val) = arg.struct_name.clone() {
                                        Some(VarStructData {struct_name: val})
                                    } else {
                                        None
                                    }
                                },
                                
                            };
                            self.add_var(arg.name.value.clone().unwrap(), var_data);
                            continue;
                        } 

                        if let Some(val) = &arg.struct_name {
                            let struct_data = self.structs.get(val).expect(&format!("no struct with name: {}",val));
                            let size = struct_data.elements.len() as u32 * struct_data.element_size;
                            self.m_stack_pos += size;
                            let pos = self.m_stack_pos;
                            self.emit(format!("    mov [rbp - {}], {}",pos, Gen::arg_pos(index,TokenType::LongType)));
                            let var_data = VarData { stack_pos: pos as i32, 
                                scope_depth: self.depth_size, 
                                var_type: arg.arg_type.token, 
                                arr_data:None, 
                                pointer_depth: 0,
                                struct_data: None,
                            
                            };
                            self.add_var(arg.name.value.clone().unwrap(), var_data);
                            continue;

                        }


                        else {
                            let pos = self.alloc(arg.arg_type.token); 
                            self.emit(format!("    mov [rbp - {}], {}",pos, Gen::arg_pos(index,arg.arg_type.token)));
                            let var_data = VarData { stack_pos: pos as i32, 
                                scope_depth: self.depth_size, 
                                var_type: arg.arg_type.token, 
                                arr_data:None, 
                                pointer_depth: 0,
                                struct_data: None,
                                
                            };
                            self.add_var(arg.name.value.clone().unwrap(), var_data);
                        }  
                        
                    }
                    self.depth_size -= 1;
                    for i in v.data {
                        self.parse_stmt(i);
                    }
                    if v.return_type.0.token == TokenType::Void {
                        self.emit("    mov rsp, rbp".to_string());
                        self.emit("    pop rbp".to_string());
                        self.emit("    ret".to_string());
                    }
                    self.m_stack_pos = self.scope_stack.pop().expect("unexcpected }") as u32;
                    self.current_func = "".to_string();

                }
                Stmt::ChangeArrElement(v) => {
                    self.eval_expr(v.expr);
                    let arr_name = v.arr_name.value.unwrap();
                    let arr = self.m_vars.get(&arr_name).expect(&format!("there no array with name: {}",arr_name));
                    let arr_type = arr.var_type;
                    let arr_stack_pos = arr.stack_pos;
                    let start_pos = arr.stack_pos;
                    let type_size: i32 = self.get_size(arr.var_type) as i32;
                    if v.element.token == TokenType::Num {
                        let elemnet: i32 = v.element.value.unwrap().parse().unwrap();
                        let element_pos = start_pos - type_size * elemnet;
                        self.emit(format!("    mov {} [rbp - {}], {}",Gen::get_word(arr.var_type),element_pos, Gen::get_rax_register(arr.var_type))); 
                    } else {
                        let index_name = v.element.value.unwrap();
                        let index_stack_pos = {
                            let iv = self.m_vars
                                .get(&index_name)
                                .expect(&format!("no var with name: {}", index_name));
                            iv.stack_pos
                        };
                        // rsi = index
                        self.emit(format!(
                            "    mov {}, {} [rbp-{}]",
                            Gen::get_rsi_regsiter(arr_type),Gen::get_word(arr_type),index_stack_pos
                        ));
                        // rsi = index * elem_size
                        self.emit(format!(
                            "    imul rsi, {}",
                            type_size
                        ));
                        // rdi = &array[0]
                        self.emit(format!(
                            "    lea rdi, [rbp-{}]",
                            arr_stack_pos
                        ));
                        // rdi = &array[index]
                        self.emit("    add rdi, rsi".to_string());

                        self.emit(format!("    mov [rdi], {}",Gen::get_rax_register(arr_type) ));
                    }


                }
                Stmt::FunctionCall(v) => {

                    let name = v.name.value.clone().unwrap();
                    let func_data = self
                        .functions
                        .get(&name)
                        .cloned();
                    if func_data.is_some() {
                        let func_data = func_data.unwrap();
                        for (index, v) in v.args.iter().enumerate() {
                            let mut expr_type = self.get_type_of_expr(v.to_vec());
                            if expr_type.1 > 0 {
                                expr_type.0 = TokenType::LongType;
                            }
                            self.eval_expr(v.to_vec());
                            self.emit(format!("    mov {}, {}",Gen::arg_pos(index, expr_type.0), Gen::get_rax_register(expr_type.0)));
                        }

                        for (index, arg_data) in func_data.args.iter().enumerate() {
                            let expr = self.get_type_of_expr(v.args[index].clone());
                            if expr.0 != arg_data.arg_type.token
                            || expr.1 != arg_data.pointer_depth {
                                self::panic!("wrong arg type pasted {:?} p_depth: {:?}\nexcpected {:?} p_depth: {:?}", expr.0,expr.1, arg_data.arg_type.token,arg_data.pointer_depth);
                            }
                        }
                        self.emit("    sub rsp,8".to_string());
                        self.emit(format!("    call {}",name));
                        self.emit("    add rsp,8".to_string());
                    }
                    else {
                        self::panic!("Trying to call unkown function: {}\n {:?}",name,self.functions);
                    }
                }
                Stmt::AsmCode(v) => {
                    for i in v.code.iter() {
                        let mut var_buf = String::new();
                        let mut buf = String::new();
                        let mut iter = i.chars();

                        while let Some(j) = iter.next() {
                            if j != '(' {
                                buf.push(j);
                            } else {
                                while let Some(next) = iter.next() {
                                    if next == ')' {
                                        break;
                                    }
                                    else {
                                        var_buf.push(next);
                                    }
                                }
                                let var = self.m_vars.get(&var_buf).expect(format!("unkown var: {}",&var_buf).as_str());
                                buf.push_str(&format!("[rbp - {}]",var.stack_pos));
                            }
                        }
                        self.emit(format!("    {}",buf));
                    }
                }
                _ => self::panic!("trying to gen unkown expr: {:?}",expr)
            }
        }
}