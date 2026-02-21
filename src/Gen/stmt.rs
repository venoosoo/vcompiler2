use crate::Gen::Gen;
use crate::Ir::r#gen::{ self, ArrData, StructData, VarData, VarStructData};
use crate::Ir::stmt::*;
use crate::Tokenizer::TokenType;

impl CreateVar {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        let expr_type = gen_helper.get_type_of_expr(&self.stmt);
        if expr_type.pointer_depth != 0 {
            panic!("trying to create var with pointer value");
        }
        gen_helper.eval_expr(&mut self.stmt);
        let pos: i32 = gen_helper.alloc(self.Type) as i32;
        gen_helper.emit(format!("    mov {} [rbp - {}], {}",Gen::get_word(self.Type),pos, Gen::get_rax_register(self.Type)));
        gen_helper.add_var(self.var.clone(), VarData { 
            stack_pos: pos, 
            scope_depth: gen_helper.depth_size, 
            var_type: self.Type, 
            arr_data: None,
            pointer_depth: expr_type.pointer_depth,
            struct_data: None
            
        });
    }
}

impl CreatePointer {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        let expr_type = gen_helper.get_type_of_expr(&self.stmt);
        gen_helper.eval_expr(&mut self.stmt);
    
    
        if self.pointer_depth != expr_type.pointer_depth && expr_type.var_type != TokenType::IntType {
            panic!("trying to create pointer var with unexcepted value");
        }

        if self.type_ != expr_type.var_type {
            panic!("trying to create pointer with wrong type value");
        }

        // pointers takes 8 bytes no matter the real type
        let pos: i32 = gen_helper.alloc(TokenType::LongType) as i32;
        gen_helper.emit(format!("    mov [rbp - {}], rax",pos));
        let var_data = VarData {
            stack_pos: pos,
            scope_depth: gen_helper.depth_size,
            var_type: self.type_,
            arr_data: None,
            pointer_depth: self.pointer_depth,
            struct_data: None

        };
        gen_helper.m_vars.insert(self.var.clone(), var_data);
    }
}


impl ChangePtrValue {
    pub fn eval(&mut self,gen_helper: &mut Gen) {
        let expr_type = gen_helper.get_type_of_expr(&self.stmt);
        gen_helper.eval_expr(&mut self.stmt);
        let var_data = gen_helper.m_vars.get(&self.var)
        .expect(&format!("no var with name: {}",self.var));
        
        let var_type = var_data.var_type;
        if var_type != expr_type.var_type && expr_type.pointer_depth != self.pointer_depth {
            panic!("when changing var: {}, unexcpected value",self.var)
        }
        gen_helper.emit(format!("    mov rsi, [rbp - {}]",var_data.stack_pos));
        gen_helper.emit(format!("    mov {} [rsi], {}",Gen::get_word(var_type), Gen::get_rax_register(var_type)));
    }
}


impl InitArray { 
    pub fn eval(&mut self, gen_helper: &mut Gen){
        let arr_size: u32 = self.size.value.as_ref().unwrap().parse().unwrap();
        let type_size = gen_helper.get_size(self.arr_type.token); 
        let alloc_size = type_size * arr_size;
        gen_helper.m_stack_pos += alloc_size;
        let stack_pos: u32 = gen_helper.m_stack_pos;
        let mut amount_taken: u32 = 0;
        if self.data.len() > arr_size as usize {
            panic!("trying to init array with more numbers than size");
        }
        for i in &self.data {
            gen_helper.emit(format!("    mov {} [rbp - {}], {}",Gen::get_word(self.arr_type.token),(stack_pos - amount_taken * type_size), i.value.as_ref().unwrap()));
            amount_taken += 1;
        }
        let arr_data = ArrData {
            size: arr_size,
        };
        let arr_var = VarData {
            stack_pos: stack_pos as i32,
            scope_depth: gen_helper.depth_size,
            var_type: self.arr_type.token,
            arr_data: Some(arr_data),
            pointer_depth: 1,
            struct_data: None
        };
        gen_helper.add_var(self.name.value.clone().unwrap(), arr_var);
    }
}

impl IfStmt {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        gen_helper.eval_expr(&mut self.expr);
        gen_helper.emit("    cmp rax, 0".to_string());
        let id = gen_helper.get_id();
        if self.else_data.len() >= 1 {
            gen_helper.emit(format!("    je else_{}",id));
        }
        else {
            gen_helper.emit(format!("    je end_if_{}",id));
        }
        gen_helper.emit(format!("if_{}:",id));
        for i in self.data.iter_mut() {
            gen_helper.parse_stmt(i);
        }
        if self.else_data.len() >= 1 {
            gen_helper.emit(format!("    je end_if_{}",id));
            gen_helper.emit(format!("else_{}:",id));
            for i in self.else_data.iter_mut() {
                gen_helper.parse_stmt( i);
            }
            
        }
        gen_helper.emit(format!("end_if_{}:",id));
    }
}


impl WhileStmt {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        let id = gen_helper.get_id();
        gen_helper.emit(format!("while_{}:",id));
        gen_helper.eval_expr(&mut self.expr);
        gen_helper.emit("    cmp rax, 1".to_string());
        gen_helper.emit(format!("    jne end_while_{}",id));
        for i in self.data.iter_mut() {
            gen_helper.parse_stmt(i);
        }
        gen_helper.emit(format!("    jmp while_{}",id));
        gen_helper.emit(format!("end_while_{}:",id));
    }
}


impl ForStmt {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        let id = gen_helper.get_id();

        // initializing the temp var as local 
        gen_helper.depth_size += 1;
        gen_helper.scope_stack.push(gen_helper.m_stack_pos as i32);
        gen_helper.parse_stmt(&mut self.expr1);
        gen_helper.depth_size -= 1;

        gen_helper.emit(format!("for_{}:",id));
        gen_helper.eval_expr(&mut self.expr2);
        gen_helper.emit("    test rax, rax".to_string());
        gen_helper.emit(format!("    je end_for_{}",id));
        for i in self.data.iter_mut() {
            gen_helper.parse_stmt(i);
        }
        gen_helper.parse_stmt(&mut self.expr3);
        gen_helper.emit(format!("    jmp for_{}",id));
        gen_helper.emit(format!("end_for_{}:",id));
        gen_helper.m_stack_pos = gen_helper.scope_stack.pop().expect("unexcpected }") as u32;
    }
}

impl IncVar {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        let var = gen_helper.m_vars.get(self.var.value.as_ref().unwrap())
        .expect(&format!("no var with name: {}",self.var.value.as_ref().unwrap()));
        let pos = var.stack_pos;
        let rax_reg = Gen::get_rax_register(var.var_type);
        gen_helper.emit(format!("    mov {} {}, [rbp - {}]",Gen::get_word(var.var_type),rax_reg,pos));
        gen_helper.emit(format!("    inc {}",rax_reg));
        gen_helper.emit(format!("    mov [rbp - {}], {}",pos,rax_reg));
    }
}

impl DecVar {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        let var = gen_helper.m_vars.get(self.var.value.as_ref().unwrap()).unwrap();
        let pos = var.stack_pos;
        gen_helper.emit(format!("    mov eax, [rbp - {}]",pos));
        gen_helper.emit("    dec eax".to_string());
        gen_helper.emit(format!("    mov [rbp - {}], eax",pos));
    }
}

impl Ret {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        let type_expr = gen_helper.get_type_of_expr(&self.expr);
        gen_helper.eval_expr(&mut self.expr);
        let func_data = gen_helper.functions.get(&self.func_name).expect(&format!("something wrong using return on unkown function: {}",self.func_name));
        
        if type_expr.var_type == func_data.return_type.var_type 
        && type_expr.pointer_depth == func_data.return_type.pointer_depth  
        {
            gen_helper.emit("    mov rsp, rbp".to_string());
            gen_helper.emit("    pop rbp".to_string());
            gen_helper.emit("    ret".to_string());
        }
        else {
            println!("type_expr: {:?}\nfunc_data: {:?}",type_expr,func_data);
            panic!("trying to return with wrong type in: {}",self.func_name);
        }
    }
}



impl ChangeStructValue {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        let expr_type = gen_helper.get_type_of_expr(&self.expr);
        let var_data = gen_helper.m_vars.get(&self.struct_name)
        .expect(&format!("no var with name: {}",self.struct_name));
        
        if let Some(val) = var_data.struct_data.as_ref() {
            let struct_data = gen_helper.structs.get(&val.struct_name)
            .expect(&format!("no struct with name: {:?}",self.struct_name));
           
            let field = struct_data.elements.get(&self.value_name)
            .expect(&format!("in var: {:?} there's no field: {:?}",self.struct_name, self.value_name));
            
            if expr_type.var_type == field.arg_type.token && expr_type.pointer_depth == field.pointer_depth {
                let stack_pos = var_data.stack_pos as u32 - (field.pos * struct_data.element_size);
                gen_helper.eval_expr(&mut self.expr);
                gen_helper.emit(format!("    mov {} [rbp - {}], {}",Gen::get_word(expr_type.var_type),stack_pos, Gen::get_rax_register(expr_type.var_type)));
            }
            else {
                panic!("trying to assign var: {} in struct {} with wrong type",self.struct_name,self.value_name)
            }
            
        }
    }
}


impl CreateStruct {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        if let Some(mut expr) = self.expr.as_mut() {
            let expr_type = gen_helper.get_type_of_expr(&expr);
            if expr_type.var_type != TokenType::Struct {
                panic!("wrong type when trying to assing to struct")
            }
            if self.pointer_depth != expr_type.pointer_depth {
                panic!("wrong pointer depth when assing to struct")
            }
            gen_helper.eval_expr(&mut expr);
            let pos = gen_helper.alloc(TokenType::LongType);
            gen_helper.emit(format!("    mov QWORD [rbp - {}], rax",pos));
        }
        if self.pointer_depth == 0 {
            let struct_data = gen_helper.structs.get(&self.struct_name).expect(&format!("no struct with name: {:?}",self.struct_name));
            gen_helper.m_stack_pos += struct_data.element_size * struct_data.elements.len() as u32;
        }
        let res = VarData {
            stack_pos: gen_helper.m_stack_pos as i32,
            scope_depth: gen_helper.depth_size,
            var_type: TokenType::Struct,
            arr_data: None,
            pointer_depth: self.pointer_depth,
            struct_data: Some(VarStructData {struct_name: self.struct_name.clone()}),
        };
        gen_helper.m_vars.insert(self.var_name.clone(), res);
    }
}


impl ChangePtrStructValue {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        let expr_type = gen_helper.get_type_of_expr(&self.expr);
        let var_data = gen_helper.m_vars.get(&self.struct_name).expect(&format!("no var with name: {}",self.struct_name));
        if let Some(val) = var_data.struct_data.as_ref() {
            let struct_pos = var_data.stack_pos;
            let struct_data = gen_helper.structs.get(&val.struct_name).expect(&format!("no struct with name: {:?}",val.struct_name));
            let element_size = struct_data.element_size;
            let field = struct_data.elements.get(&self.value_name).expect(&format!("in var: {:?} there's no field: {:?}",self.struct_name, self.value_name));
            let field_pos = field.pos;
            if expr_type.var_type == field.arg_type.token && expr_type.pointer_depth == field.pointer_depth {
                gen_helper.emit(format!("    mov rsi, [rbp - {}]",struct_pos));
                gen_helper.emit(format!("    add rsi, {}",field_pos * element_size));
                gen_helper.eval_expr(&mut self.expr);
                gen_helper.emit(format!("    mov [rsi], rax"));

            }
            else {
                panic!("trying to assign var: {} in struct {} with wrong type",self.struct_name,self.value_name)
            }
            
        }
    }
}



impl InitFunc {


    fn get_args_size(&self, args: &Vec<Arg>, gen_helper: &mut Gen) -> u32 {
        let mut res = 0;
        for arg in args {
            res += self.get_arg_size(arg, gen_helper);
        }
        res = (res + 15) & !15;
        res
    }

    fn get_arg_size(&self, arg: &Arg, gen_helper: &mut Gen) -> u32 {
        if arg.pointer_depth > 0 {
            return 8;
        }
        else if let Some(val) = &arg.struct_name {
            let struct_data = gen_helper.structs.get(val).expect(&format!("no struct with name: {}",val));
            let size = struct_data.elements.len() as u32 * struct_data.element_size;
            size
        }
        else {
            gen_helper.get_size(arg.arg_type.token)
        }

    }

    pub fn eval(&mut self, gen_helper: &mut Gen) {
        gen_helper.emit(format!("{}:",self.name.value.as_ref().unwrap()));
        let stmt_stack_size = gen_helper.calc_stack_size(&self.data);

        let total = self.get_args_size(&self.args, gen_helper) + stmt_stack_size;
        gen_helper.emit("    push rbp".to_string());
        gen_helper.emit("    mov rbp, rsp".to_string());
        gen_helper.emit(format!("    sub rsp, {}",total));
        // so arg var will be local to the func
        gen_helper.depth_size += 1;
        gen_helper.scope_stack.push(gen_helper.m_stack_pos as i32);
        for (index, arg) in self.args.iter().enumerate() {
            let mut pos = 0;
            let mut arg_type = arg.arg_type.token;
            if arg.pointer_depth > 0 {
                arg_type = TokenType::LongType;
            } 
            if let Some(val) = &arg.struct_name {
                let struct_data = gen_helper.structs.get(val).expect(&format!("no struct with name: {}",val));
                let size = struct_data.elements.len() as u32 * struct_data.element_size;
                gen_helper.m_stack_pos += size;
                pos = gen_helper.m_stack_pos;

            }
            else  {
                pos = gen_helper.alloc(arg_type); 
            }
            gen_helper.emit(format!("    mov [rbp - {}], {}",pos, Gen::arg_pos(index,arg_type)));
            let var_data = VarData { stack_pos: pos as i32, 
                scope_depth: gen_helper.depth_size, 
                var_type: arg.arg_type.token, 
                arr_data:None, 
                pointer_depth: arg.pointer_depth,
                struct_data:  {
                    if let Some(val) = arg.struct_name.clone() {
                        Some(VarStructData { struct_name: val } )
                    }
                    else {
                        None
                    }
                },
                
            };
            gen_helper.add_var(arg.name.value.clone().unwrap(), var_data);  
            
        }
        gen_helper.depth_size -= 1;
        for i in self.data.iter_mut() {
            gen_helper.parse_stmt(i);
        }
        if self.return_type.var_type == TokenType::Void {
            gen_helper.emit("    mov rsp, rbp".to_string());
            gen_helper.emit("    pop rbp".to_string());
            gen_helper.emit("    ret".to_string());
        }
        gen_helper.m_stack_pos = gen_helper.scope_stack.pop().expect("unexcpected }") as u32;
        gen_helper.current_func = "".to_string();
    }
}



impl ChangeArrElement {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        gen_helper.eval_expr(&mut self.expr);


        let (arr_type, arr_stack_pos, type_size) = {
            let arr = gen_helper.m_vars.get(self.arr_name.value.as_ref().unwrap()).expect(&format!("there no array with name: {}",self.arr_name.value.as_ref().unwrap()));
            let arr_type = arr.var_type;
            let arr_stack_pos = arr.stack_pos;
            let type_size: i32 = gen_helper.get_size(arr.var_type) as i32;
            (arr_type, arr_stack_pos,type_size)
        };

        if self.element.token == TokenType::Num {
            let elemnet: i32 = self.element.value.as_ref().unwrap().parse().unwrap();
            let element_pos = arr_stack_pos - type_size * elemnet;
            gen_helper.emit(format!("    mov {} [rbp - {}], {}",Gen::get_word(arr_type),element_pos, Gen::get_rax_register(arr_type))); 
        } else {
            let index_stack_pos = {
                let iv = gen_helper.m_vars
                    .get(self.element.value.as_ref().unwrap())
                    .expect(&format!("no var with name: {}", self.element.value.as_ref().unwrap()));
                iv.stack_pos
            };
            // rsi = index
            gen_helper.emit(format!(
                "    mov {}, {} [rbp-{}]",
                Gen::get_rsi_regsiter(arr_type),Gen::get_word(arr_type),index_stack_pos
            ));
            // rsi = index * elem_size
            gen_helper.emit(format!(
                "    imul rsi, {}",
                type_size
            ));
            // rdi = &array[0]
            gen_helper.emit(format!(
                "    lea rdi, [rbp-{}]",
                arr_stack_pos
            ));
            // rdi = &array[index]
            gen_helper.emit("    add rdi, rsi".to_string());

            gen_helper.emit(format!("    mov [rdi], {}",Gen::get_rax_register(arr_type) ));
        }
    }
}

impl FunctionCall {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        let name = self.name.value.as_ref().unwrap();
        let func_data = gen_helper
            .functions
            .get(name)
            .cloned();
        if func_data.is_some() {
            let func_data = func_data.unwrap();
            for (index,  mut v) in self.args.iter_mut().enumerate() {
                let mut expr_type = gen_helper.get_type_of_expr(v);
                if expr_type.pointer_depth > 0 {
                    expr_type.var_type = TokenType::LongType;
                }
                gen_helper.eval_expr(  &mut v);
                gen_helper.emit(format!("    mov {}, {}",Gen::arg_pos(index, expr_type.var_type), Gen::get_rax_register(expr_type.var_type)));
            }

            for (index, arg_data) in func_data.args.iter().enumerate() {
                let expr = gen_helper.get_type_of_expr(&self.args[index]);
                if expr.var_type != arg_data.arg_type.token
                || expr.pointer_depth != arg_data.pointer_depth {
                    panic!("wrong arg type pasted {:?} p_depth: {:?}\nexcpected {:?} p_depth: {:?}", expr.var_type,expr.pointer_depth, arg_data.arg_type.token,arg_data.pointer_depth);
                }
            }
            gen_helper.emit("    sub rsp,8".to_string());
            gen_helper.emit(format!("    call {}",name));
            gen_helper.emit("    add rsp,8".to_string());
        }
        else {
            panic!("Trying to call unkown function: {}\n {:?}",name,gen_helper.functions);
        }
    }
}



impl AsmCode {
    pub fn eval(&mut self, gen_helper: &mut Gen) {
        for i in self.code.iter() {
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
                    let var = gen_helper.m_vars.get(&var_buf).expect(format!("unkown var: {}",&var_buf).as_str());
                    buf.push_str(&format!("[rbp - {}]",var.stack_pos));
                }
            }
            gen_helper.emit(format!("    {}",buf));
        }
    }
}