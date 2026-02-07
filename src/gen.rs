use core::panic;
use std::{collections::HashMap, fmt::Write};

use crate::parser::{ FunctionCall, Operator, PushNum, PushVar, Ret, RpnExpr};
use crate::tokenizer::Token;
use crate::{parser::Expr, tokenizer::TokenType, parser::Arg};
#[derive(Debug)]
struct expr_stack {
    reg: String,
    var_typd: (TokenType, u32),
}
#[derive(Debug)]
// ????????
struct arr_data {
    size: u32,


}
#[derive(Debug)]
struct var_data {
    stack_pos: i32,
    scope_depth: usize,
    var_type: TokenType,
    arr_data: Option<arr_data>,
    pointer_depth: u32,
}
#[derive(Debug, Clone)]
struct func_data {
    args: Vec<Arg>,
    // return type and pointer depth
    return_type: (Token, u32),
}

pub struct Gen {
    m_ast: Vec<Expr>,
    m_vars: HashMap<String,var_data>,
    m_out: String,
    depth_size: usize,
    scope_stack: Vec<i32>,
    m_stack_pos: u32,
    functions: HashMap<String, func_data>,
    current_func: String,
    id: usize,
}




impl Gen {


    pub fn new(m_ast: Vec<Expr>) -> Gen {
        Gen {
            m_ast,
            m_vars: HashMap::new(),
            m_out: String::new(),
            depth_size: 0,
            scope_stack: Vec::new(),
            m_stack_pos: 0,
            functions: HashMap::new(),
            current_func: String::new(),
            id: 0,
        }
    }


    fn emit(&mut self,s: String) {
        writeln!(self.m_out, "{}", s);
    }

    fn get_id(&mut self) -> usize {
        self.id += 1;
        self.id
    }

    fn get_size(&self, token: TokenType) -> u32 {
        match token {
            TokenType::IntType => 4,
            TokenType::CharType => 1,
            TokenType::ShortType => 2,
            TokenType::LongType => 8,
            _ => panic!("trying to get size of unexpected type: {:?}",token),
        }
    }

    fn alloc(&mut self, ty: TokenType) -> u32 {
        let size = self.get_size(ty);
        self.m_stack_pos += size;
        self.m_stack_pos
    }


    fn calc_stack_size(&self,exprs: &[Expr]) -> u32 {
        let total: u32 = exprs
            .iter()
            .map(|e| match e {
                Expr::Var(v) => self.get_size(v.Type),
                Expr::InitArray(v) => {
                    let arr_size: u32 = v.size.value.clone().unwrap().parse().unwrap();
                    self.get_size(v.arr_type.token) * arr_size
                },
                Expr::IfStmt(v) => {
                    std::cmp::max(self.calc_stack_size(&v.data), self.calc_stack_size(&v.else_data))
                }
                Expr::WhileStmt(v) => {
                    self.calc_stack_size(&v.data)
                }

                Expr::ForStmt(v) => {
                    self.calc_stack_size(&v.data)
                }
                _ => {
                    0
                },
            })
            .sum();
        total
    }

    pub fn gen_asm(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        self.emit("section .text".to_string());
        self.emit("global _start".to_string());
        self.emit("_start:".to_string());
        self.emit("    sub rsp, 8".to_string());
        self.emit("    call main".to_string());
        self.emit("    add rsp, 8".to_string());
        self.emit("    mov rax, 60".to_string());
        self.emit("    xor rdi, rdi".to_string());
        self.emit("    syscall".to_string());
        self.parse_expr()?;
        Ok(self.m_out.clone())
    }


    fn is_num(token: TokenType) -> bool {
        match token {
            TokenType::IntType => true,
            TokenType::LongType => true,
            TokenType::ShortType => true,
            _ => false,
        }
    } 

    fn get_rax_register(token: TokenType) -> String {
        match token {
            TokenType::IntType => "eax".to_string(),
            TokenType::ShortType => "ax".to_string(),
            TokenType::LongType => "rax".to_string(),
            TokenType::CharType => "al".to_string(),
            _ => panic!("not a type: {:?}", token),
        }
    }

    fn get_rsi_regsiter(token: TokenType) -> String {
        match token {
            TokenType::IntType => "esi".to_string(),
            TokenType::ShortType => "si".to_string(),
            TokenType::LongType => "rsi".to_string(),
            TokenType::CharType => "sil".to_string(),
            _ => panic!("not a type: {:?}", token),
        }
    }

    fn get_rbx_register(token: TokenType) -> String {
        match token {
            TokenType::IntType => "ebx".to_string(),
            TokenType::ShortType => "bx".to_string(),
            TokenType::LongType => "rbx".to_string(),
            TokenType::CharType => "bl".to_string(),
            _ => panic!("not a type: {:?}", token),
        }
    }

    fn convert_reg_to_size(reg: &str) -> u32 {
        match reg {
            "rax" => 8,
            "eax" => 4,
            "ax" => 2,
            "al" => 1,
            "rbx" => 8,
            "ebx" => 4,
            "bx" => 2,
            "bl" => 1,
            __ => panic!("unkown reg at convert_reg_to_size: {}",reg),
        }
        
    }

    fn convert_size_to_type(size: u32) -> TokenType {
        match size {
            1 => TokenType::CharType,
            2 => TokenType::ShortType,
            4 => TokenType::IntType,
            8 => TokenType::LongType,
            _ => panic!("unknown size in convert_size_to_type")
        }
    }


    fn compare_reg(&mut self, lhs: &String, rhs:&String) -> (String, String) {
        if lhs.starts_with('[') || rhs.starts_with('[') {
            return (lhs.to_string(),rhs.to_string());
        }
        let lhs_size = Gen::convert_reg_to_size(&lhs);
        let rhs_size = Gen::convert_reg_to_size(&rhs);



        if lhs_size < rhs_size {
            let lhs_type = Gen::convert_size_to_type(rhs_size);
            let correct_reg = Gen::get_rax_register(lhs_type);
            if lhs_size == 4 {
                self.emit(format!("    movsxd {}, {}",correct_reg , lhs));
            } else {
                self.emit(format!("    movsx {}, {}",correct_reg , lhs));
            }
            return (correct_reg, rhs.to_string());
        }
        if rhs_size < lhs_size {
            let rhs_type = Gen::convert_size_to_type(lhs_size);
            let correct_reg = Gen::get_rbx_register(rhs_type);
            if rhs_size == 4 {
                self.emit(format!("    movsxd {}, {}",correct_reg , rhs));
            } else {
                self.emit(format!("    movsx {}, {}",correct_reg , rhs));
            }
            return (lhs.to_string(), correct_reg);
        }
        return (lhs.to_string(),rhs.to_string());
    }

    fn calc_expr_stack_size(stack: &Vec<expr_stack>) -> u32 {
        let mut res = 0u32;
        for reg in stack {
            res += Gen::convert_reg_to_size(&reg.reg);
        }
        res
    }

    fn eval_expr(&mut self, rpn: Vec<RpnExpr>) {
        let mut stack: Vec<expr_stack> = Vec::new();
        for expr in rpn {
            match expr {
                RpnExpr::PushNum(p) => {
                    let val = p.data.value.clone().unwrap();
                    
                    if stack.is_empty() {
                        self.emit(format!("    mov rax, {}",val));
                        stack.push(expr_stack { reg: "rax".to_string(), var_typd: (TokenType::Num, 0) })
                    } else if stack.len() == 1 {
                        self.emit(format!("    mov rbx, {}", val));
                        stack.push(expr_stack { reg: "rbx".to_string(), var_typd: (TokenType::Num, 0) })
                    } else {
                        let slot = Gen::calc_expr_stack_size(&stack) + 8;
                        self.emit(format!(
                            "    mov QWORD [rbp-{}], {}",
                            slot, val
                        ));
                        stack.push(expr_stack { reg: format!("[rbp-{}]", slot), var_typd: (TokenType::Num, 0) })
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
                    stack.push(expr_stack { reg: "rax".to_string(), var_typd: (var_type, pointer_depth) })

                }


                RpnExpr::GetAddr(v) => {
                    let var_name = v.var.value.unwrap();
                    let var_data = self.m_vars.get(&var_name).expect(&format!("no var with name: {}",var_name));
                    let pointer_depth = var_data.pointer_depth;
                    let var_type = var_data.var_type;
                    self.emit(format!("    lea rsi, [rbp-{}]",var_data.stack_pos));
                    if stack.is_empty() {
                        self.emit(format!("    mov rax, rsi"));
                        stack.push(expr_stack { reg: "rax".to_string(), var_typd: (var_type, pointer_depth + 1) })
                    } else if stack.len() == 1 {
                        self.emit(format!("    mov rbx, rsi"));
                        stack.push(expr_stack { reg: "rbx".to_string(), var_typd: (var_type, pointer_depth + 1) })
                    } else {
                        let slot = Gen::calc_expr_stack_size(&stack) + 8;
                        self.emit(format!(
                            "    mov QWORD [rbp-{}], rsi",
                            slot
                        ));
                        stack.push(expr_stack { reg: format!("[rbp-{}]", slot), var_typd: (var_type, pointer_depth + 1) })
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
                    let var: &var_data = self.m_vars.get(&name).expect(format!("unkown var: {}",&name).as_str());
                    let pos = var.stack_pos;
                    let var_type = var.var_type;
                    let pointer_depth = var.pointer_depth;

                    
                    if stack.is_empty() {
                        let mut arg = Gen::get_rax_register(var.var_type);
                        if var.pointer_depth != 0 {
                            arg = "rax".to_string();
                        }
                        if var.arr_data.is_some() {
                            self.emit(format!("    lea rax, [rbp-{}]",pos));
                            stack.push(expr_stack { reg: format!("{}",arg), var_typd: (var_type,pointer_depth) });
                        }
                        else {
                            self.emit(format!("    mov {}, [rbp-{}]",arg ,pos));
                            stack.push(expr_stack { reg: format!("{}",arg), var_typd: (TokenType::Num,0) });
                        }
                    } else if stack.len() == 1 {
                        let mut arg = Gen::get_rbx_register(var.var_type);
                        if var.pointer_depth != 0 {
                            arg = "rbx".to_string();
                        }
                        if var.arr_data.is_some() {
                            self.emit(format!("    lea rbx, [rbp-{}]",pos));
                        }
                        else {
                            self.emit(format!("    mov {}, [rbp-{}]",arg ,pos));
                        }
                        stack.push(expr_stack { reg: format!("{}",arg), var_typd: (TokenType::Num,0) });
                    } else {
                        let slot = Gen::calc_expr_stack_size(&stack) + 8;
                        if var.arr_data.is_some() {
                            self.emit(format!("    lea [rbp - {}], [rbp - {}]",slot,pos));
                        }
                        else {
                            self.emit(format!("    mov [rbp - {}], [rbp-{}]",slot ,pos));
                        }
                        stack.push(expr_stack { reg: format!("[rbp-{}]", slot), var_typd: (TokenType::Num,0) });
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
                            stack.push(expr_stack { reg: format!("{}",lhs.reg), var_typd: (TokenType::Num,0) });
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
                            
                            stack.push(expr_stack { reg: "rax".into(), var_typd: (TokenType::Num,0) });
                        }
                        
                        _ => panic!("unsupported operator {:?}", t),
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


    fn parse_expr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for i in &self.m_ast {
            match i {
                Expr::InitFunc(v) => {
                    let name = v.name.value.clone().unwrap();
                    let res = func_data {
                        return_type: v.return_type.clone(),
                        args: v.args.clone()
                    };
                    self.functions.insert(name, res);
                }
                _ => continue,
            }
        }

        for i in 0..self.m_ast.len() {
            let expr = self.m_ast[i].clone();
            self.parse_one_expr(expr);

        }
        Ok(())
    }


    fn arg_pos(pos: usize, token: TokenType) -> String {
        match token {
            TokenType::IntType => match pos {
                0 => "edi".to_string(),
                1 => "esi".to_string(),
                2 => "edx".to_string(),
                3 => "r10d".to_string(),  // r10 32-bit
                4 => "r8d".to_string(),   // r8 32-bit
                _ => panic!("arg_pos unknown arg: {}", pos),
            },
            TokenType::LongType => match pos {
                0 => "rdi".to_string(),
                1 => "rsi".to_string(),
                2 => "rdx".to_string(),
                3 => "r10".to_string(),
                4 => "r8".to_string(),
                _ => panic!("arg_pos unknown arg: {}", pos),
            },
            TokenType::ShortType => match pos {
                0 => "di".to_string(),
                1 => "si".to_string(),
                2 => "dx".to_string(),
                3 => "r10w".to_string(),
                4 => "r8w".to_string(),
                _ => panic!("arg_pos unknown arg: {}", pos),
            },
            TokenType::CharType => match pos {
                0 => "dil".to_string(),
                1 => "sil".to_string(),
                2 => "dl".to_string(),
                3 => "r10b".to_string(),
                4 => "r8b".to_string(),
                _ => panic!("arg_pos unknown arg: {}", pos),
            },
            _ => panic!("unknown arg_pos token: {:?}", token),
        }
    }


    fn get_type_of_expr(&self,expr: Vec<RpnExpr>) -> (TokenType, u32) {
        let mut res = TokenType::IntType; // default one
        let mut pointer_depth = 0;
        for i in expr { 
            match i {

                RpnExpr::PushVar(v) => {
                    if v.data.token == TokenType::Var {
                        let name = v.data.value.unwrap();
                        let var = self.m_vars.get(&name).expect(&format!("no variable with name: {}",name));
                        if self.get_size(res) < self.get_size(var.var_type) {
                            res = v.data.token;
                        }
                        if var.pointer_depth > pointer_depth {
                            pointer_depth = var.pointer_depth;
                        }
                    }
                    else if self.get_size(res) < self.get_size(v.data.token) {
                        res = v.data.token;
                    }
                }
                RpnExpr::GetAddr(v) => {
                    let name = v.var.value.unwrap();
                    let var_data = self.m_vars.get(&name).expect(&format!("no var with name: {}",name));
                    res = var_data.var_type;
                    pointer_depth = var_data.pointer_depth + 1;
                }

                RpnExpr::Deref(v) => {
                    let name = v.var.value.unwrap();
                    let var_data = self.m_vars.get(&name).expect(&format!("no var with name: {}",name));
                    res = var_data.var_type;
                    pointer_depth = var_data.pointer_depth - v.stack_depth;
                }

                _ => continue,
            }
        }
        (res, pointer_depth)
    }


    fn get_word(token: TokenType) -> String {
        match token {
            TokenType::IntType => "DWORD".to_string(),
            TokenType::ShortType => "WORD".to_string(),
            TokenType::LongType => "QWORD".to_string(),
            TokenType::CharType => "BYTE".to_string(),
            _ => panic!("not a type: {:?}", token),
        }
    }

    fn add_var(&mut self,key: String, value: var_data) {
        if self.m_vars.contains_key(&key) {
            panic!("redefinition of variable: {}",key);
        }
        self.m_vars.insert(key, value);
    }


    fn parse_one_expr(&mut self,expr: Expr) {
        match expr {
                Expr::Var(v) => {
                    let expr_type = self.get_type_of_expr(v.Expr.clone());
                    if expr_type.1 != 0 {
                        panic!("trying to create var with pointer value");
                    }
                    self.eval_expr(v.Expr);
                    let pos: i32 = self.alloc(v.Type) as i32;
                    self.emit(format!("    mov {} [rbp - {}], {}",Gen::get_word(v.Type),pos, Gen::get_rax_register(v.Type)));
                    self.add_var(v.Var, var_data { 
                        stack_pos: pos, 
                        scope_depth: self.depth_size, 
                        var_type: v.Type, 
                        arr_data: None,
                        pointer_depth: expr_type.1,
                     
                    });
                }
                Expr::OpenScope(v) => {
                    self.scope_stack.push(self.m_stack_pos as i32);
                    self.depth_size += 1
                }
                Expr::CloseScope(v) => {
                    self.m_stack_pos = self.scope_stack.pop().expect("unexcpected }") as u32;
                    self.m_vars.retain(|_, value| {
                        value.scope_depth != self.depth_size
                    });
                    self.depth_size -= 1;
                    
                }

                Expr::CreatePointer(v) => {
                    let expr_type = self.get_type_of_expr(v.Expr.clone());
                    self.eval_expr(v.Expr);
                
                
                    if v.pointer_depth != expr_type.1 && expr_type.0 != TokenType::IntType {
                        println!("name: {}",v.Var);
                        println!("lhs: {}, rhs: {}",v.pointer_depth,expr_type.1);
                        panic!("trying to create pointer var with unexcepted value");
                    }

                    if v.Type != expr_type.0 {
                        panic!("trying to create pointer with wrong type value");
                    }
                    

                    // pointers takes 8 bytes no matter the real type
                    let pos: i32 = self.alloc(TokenType::LongType) as i32;
                    self.emit(format!("    mov [rbp - {}], rax",pos));
                    let var_data = var_data {
                        stack_pos: pos,
                        scope_depth: self.depth_size,
                        var_type: v.Type,
                        arr_data: None,
                        pointer_depth: v.pointer_depth,

                    };
                    self.m_vars.insert(v.Var, var_data);
                    
                }

                Expr::ChangePtrValue(v) => {
                    let expr_type = self.get_type_of_expr(v.Expr.clone());
                    self.eval_expr(v.Expr);
                    let var_data = self.m_vars.get(&v.Var).expect(&format!("no var with name: {}",v.Var));
                    let var_type = var_data.var_type;
                    if var_type != expr_type.0 && expr_type.1 != v.pointer_depth {
                        panic!("when changing var: {}, unexcpected value",v.Var)
                    }
                    self.emit(format!("    mov rsi, [rbp - {}]",var_data.stack_pos));
                    self.emit(format!("    mov {} [rsi], {}",Gen::get_word(var_type), Gen::get_rax_register(var_type)));
                }

                Expr::InitArray(v) => {
                    let arr_size: u32 =v.size.value.unwrap().parse().unwrap();
                    let type_size = self.get_size(v.arr_type.token); 
                    let alloc_size = type_size * arr_size;
                    self.m_stack_pos += alloc_size;
                    let stack_pos: u32 = self.m_stack_pos.try_into().unwrap();
                    let mut amount_taken: u32 = 0;
                    if v.data.len() > arr_size as usize {
                        panic!("trying to init array with more numbers than size");
                    }
                    for i in v.data {
                        self.emit(format!("    mov {} [rbp - {}], {}",Gen::get_word(v.arr_type.token),(stack_pos - amount_taken * type_size), i.value.unwrap()));
                        amount_taken += 1;
                    }
                    let arr_data = arr_data {
                        size: arr_size,
                    };
                    let arr_var = var_data {
                        stack_pos: stack_pos as i32,
                        scope_depth: self.depth_size,
                        var_type: v.arr_type.token,
                        arr_data: Some(arr_data),
                        pointer_depth: 1,
                    };
                    self.add_var(v.name.value.unwrap(), arr_var);

                }

                Expr::ChangeVar(v) => {
                    self.eval_expr(v.Expr);
                    let var = self.m_vars.get(&v.var).unwrap();
                    self.emit(format!("    mov {} [rbp - {}], {}",Gen::get_word(var.var_type),var.stack_pos, Gen::get_rax_register(var.var_type)));   
                }
                
                Expr::IfStmt(v) => {
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
                        self.parse_one_expr(i);
                    }
                    if v.else_data.len() >= 1 {
                        self.emit(format!("    je end_if_{}",id));
                        self.emit(format!("else_{}:",id));
                        for i in v.else_data {
                            self.parse_one_expr(i);
                        }
                        
                    }
                    self.emit(format!("end_if_{}:",id));
                    
                }
                Expr::WhileStmt(v) => {
                    let id = self.get_id();
                    self.emit(format!("while_{}:",id));
                    self.eval_expr(v.expr);
                    self.emit("    cmp rax, 1".to_string());
                    self.emit(format!("    jne end_while_{}",id));
                    for i in v.data {
                        self.parse_one_expr(i);
                    }
                    self.emit(format!("    jmp while_{}",id));
                    self.emit(format!("end_while_{}:",id));
                }
                Expr::ForStmt(v) => {
                    let id = self.get_id();
                    self.depth_size += 1;
                    self.scope_stack.push(self.m_stack_pos as i32);
                    self.parse_one_expr(*v.expr1.clone());
                    self.depth_size -= 1;
                    self.emit(format!("for_{}:",id));
                    self.eval_expr(v.expr2);
                    self.emit("    test rax, rax".to_string());
                    self.emit(format!("    je end_for_{}",id));
                    for i in v.data {
                        self.parse_one_expr(i);
                    }
                    self.parse_one_expr(*v.expr3);
                    self.emit(format!("    jmp for_{}",id));
                    self.emit(format!("end_for_{}:",id));
                    self.m_stack_pos = self.scope_stack.pop().expect("unexcpected }") as u32;

                }
                Expr::IncVar(v) => {
                    let var = self.m_vars.get(&v.var.value.unwrap()).unwrap();
                    let pos = var.stack_pos;
                    let rax_reg = Gen::get_rax_register(var.var_type);
                    self.emit(format!("    mov {} {}, [rbp - {}]",Gen::get_word(var.var_type),rax_reg,pos));
                    self.emit(format!("    inc {}",rax_reg));
                    self.emit(format!("    mov [rbp - {}], {}",pos,rax_reg));
                }
                Expr::DecVar(v) => {
                    let var = self.m_vars.get(&v.var.value.unwrap()).unwrap();
                    let pos = var.stack_pos;
                    self.emit(format!("    mov eax, [rbp - {}]",pos));
                    self.emit("    dec eax".to_string());
                    self.emit(format!("    mov [rbp - {}], eax",pos));
                }
                Expr::Ret(v) => {
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
                        panic!("trying to return with wrong type in: {}",v.func_name);
                    }
                }
                Expr::InitFunc(v) => {
                    let name = v.name.value.clone().unwrap();
                    self.current_func = name.clone();
                    self.emit(format!("{}:",name));
                    let mut temp_stack_size = self.calc_stack_size(&v.data);
                    for arg in &v.args {
                        if arg.pointer_depth > 0 {
                            temp_stack_size += 8;
                        } else {

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
                            let var_data = var_data { stack_pos: pos as i32, scope_depth: self.depth_size, var_type: arg.arg_type.token, arr_data:None, pointer_depth: arg.pointer_depth };
                            self.add_var(arg.name.value.clone().unwrap(), var_data);

                        } else {
                            let pos = self.alloc(arg.arg_type.token);  
                            self.emit(format!("    mov [rbp - {}], {}",pos, Gen::arg_pos(index,arg.arg_type.token)));
                            let var_data = var_data { stack_pos: pos as i32, scope_depth: self.depth_size, var_type: arg.arg_type.token, arr_data:None, pointer_depth: 0 };
                            self.add_var(arg.name.value.clone().unwrap(), var_data);
                        }  
                        
                    }
                    self.depth_size -= 1;
                    for i in v.data {
                        self.parse_one_expr(i);
                    }
                    if v.return_type.0.token == TokenType::Void {
                        self.emit("    mov rsp, rbp".to_string());
                        self.emit("    pop rbp".to_string());
                        self.emit("    ret".to_string());
                    }
                    self.m_stack_pos = self.scope_stack.pop().expect("unexcpected }") as u32;
                    self.current_func = "".to_string();

            
                }
                Expr::ChangeArrElement(v) => {
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
                Expr::FunctionCall(v) => {
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
                                panic!("wrong arg type pasted {:?} p_depth: {:?}\nexcpected {:?} p_depth: {:?}", expr.0,expr.1, arg_data.arg_type.token,arg_data.pointer_depth);
                            }
                        }
                        self.emit("    sub rsp,8".to_string());
                        self.emit(format!("    call {}",name));
                        self.emit("    add rsp,8".to_string());
                    }
                    else {
                        panic!("Trying to call unkown function: {}\n {:?}",name,self.functions);
                    }
                }
                Expr::AsmCode(v) => {
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
                _ => panic!("trying to gen unkown expr")
            }
        }
    }
