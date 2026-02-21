use core::panic;
use std::rc::Rc;
use std::{collections::HashMap, fmt::Write};

use crate::Ir::expr::RpnExpr;
use crate::Ir::Stmt;
use crate::Ir::r#gen::*;
use crate::Ir::stmt::StructArg;
use crate::Ir::stmt::TypeInfo;
use crate::Tokenizer::TokenType;


mod gen_expr;
mod gen_stmt;
mod expr;
mod stmt;

pub struct Gen {
    m_ast: Vec<Stmt>,
    m_vars: HashMap<String,VarData>,
    m_out: String,
    depth_size: usize,
    scope_stack: Vec<i32>,
    m_stack_pos: u32,
    structs: HashMap<String, StructData>,
    functions: HashMap<String, FuncData>,
    current_func: String,
    id: usize,
}




impl Gen {


    pub fn new(m_ast: Vec<Stmt>) -> Gen {
        Gen {
            m_ast,
            m_vars: HashMap::new(),
            m_out: String::new(),
            depth_size: 0,
            scope_stack: Vec::new(),
            m_stack_pos: 0,
            structs: HashMap::new(),
            functions: HashMap::new(),
            current_func: String::new(),
            id: 0,
        }
    }


    fn emit(&mut self,s: String) {
        let _ = writeln!(self.m_out, "{}", s);
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
        let size: u32 = self.get_size(ty);
        self.m_stack_pos += size;
        self.m_stack_pos
    }


    fn calc_stack_size(&self,exprs: &[Stmt]) -> u32 {
        let total: u32 = exprs
            .iter()
            .map(|e| match e {
                Stmt::CreateVar(v) => self.get_size(v.Type),

                Stmt::CreateStruct(v) => {
                    let struct_data = self.structs.get(&v.struct_name).expect(&format!("no struct with name: {:?}",v.struct_name));
                    let size = struct_data.elements.len() as u32 * struct_data.element_size;
                    size
                }

                Stmt::InitArray(v) => {
                    let arr_size: u32 = v.size.value.clone().unwrap().parse().unwrap();
                    self.get_size(v.arr_type.token) * arr_size
                },
                Stmt::IfStmt(v) => {
                    std::cmp::max(self.calc_stack_size(&v.data), self.calc_stack_size(&v.else_data))
                }
                Stmt::WhileStmt(v) => {
                    self.calc_stack_size(&v.data)
                }

                Stmt::ForStmt(v) => {
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
        self.gen_stmts()?;
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


    fn get_rdx_register(token: TokenType) -> String {
        match token {
            TokenType::IntType => "edx".to_string(),
            TokenType::ShortType => "dx".to_string(),
            TokenType::LongType => "rdx".to_string(),
            TokenType::CharType => "dl".to_string(),
            TokenType::Struct => "rdx".to_string(),
            _ => panic!("not a type: {:?}", token),
        }
    }

    fn get_rax_register(token: TokenType) -> String {
        match token {
            TokenType::IntType => "eax".to_string(),
            TokenType::ShortType => "ax".to_string(),
            TokenType::LongType => "rax".to_string(),
            TokenType::CharType => "al".to_string(),
            TokenType::Struct => "rax".to_string(),
            _ => panic!("not a type: {:?}", token),
        }
    }

    fn get_rsi_regsiter(token: TokenType) -> String {
        match token {
            TokenType::IntType => "esi".to_string(),
            TokenType::ShortType => "si".to_string(),
            TokenType::LongType => "rsi".to_string(),
            TokenType::CharType => "sil".to_string(),
            TokenType::Struct => "rsi".to_string(),
            _ => panic!("not a type: {:?}", token),
        }
    }

    fn get_rbx_register(token: TokenType) -> String {
        match token {
            TokenType::IntType => "ebx".to_string(),
            TokenType::ShortType => "bx".to_string(),
            TokenType::LongType => "rbx".to_string(),
            TokenType::CharType => "bl".to_string(),
            TokenType::Struct => "rbx".to_string(),
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

    fn calc_expr_stack_size(stack: &Vec<ExprStack>) -> u32 {
        let mut res = 0u32;
        for reg in stack {
            res += Gen::convert_reg_to_size(&reg.reg);
        }
        res
    }

    fn get_struct_element_size(&self, elements: &HashMap<String,StructArg>) -> u32 {
        let mut largest_el_size = 0;
        for (_name,i) in elements {
            if i.pointer_depth > 0 {
                largest_el_size = 8;
            }
            if self.get_size(i.arg_type.token) > largest_el_size {
                largest_el_size = self.get_size(i.arg_type.token)
            }
        }
        return largest_el_size
    }


    fn gen_stmts(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for i in self.m_ast.iter() {
            match i {
                Stmt::InitFunc(v) => {
                    let name = v.name.value.clone().unwrap();
                    let res = FuncData {
                        return_type: v.return_type.clone(),
                        args: v.args.clone()
                    };
                    self.functions.insert(name, res);
                }
                Stmt::InitStruct(v) => {
                    let size = self.get_struct_element_size(&v.elements);
                    self.structs.insert(v.name.clone(), StructData { elements: v.elements.clone(), element_size: size });

                }
                _ => continue,
            }
        }
        
        let mut ast = std::mem::take(&mut self.m_ast);
        for i in ast.iter_mut() {
            self.parse_stmt(i);
        }
        Ok(())
    }


    fn arg_pos(pos: usize, token: TokenType) -> String {
        match token {
            TokenType::IntType => match pos {
                0 => "edi".to_string(),
                1 => "esi".to_string(),
                2 => "edx".to_string(),
                3 => "r10d".to_string(),
                4 => "r8d".to_string(),
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


    fn get_type_of_expr(&self,expr: &Vec<RpnExpr>) -> TypeInfo {
        let mut res = TokenType::IntType; // default one
        let mut pointer_depth = 0;
        let expr_len = expr.len();
        for i in expr { 
            match i {

                RpnExpr::PushVar(v) => {
                    if v.data.token == TokenType::Var {
                        let name = v.data.value.as_ref().unwrap();
                        let var = self.m_vars.get(name).expect(&format!("no variable with name: {}",name));

                        if var.struct_data.is_some() {
                            panic!("cannot copy a struct");
                        }
                        
                        if self.get_size(res) < self.get_size(var.var_type) || expr_len < 2 {
                            res = var.var_type;
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
                    let name = v.var.value.as_ref().unwrap();
                    let var_data = self.m_vars.get(name).expect(&format!("no var with name: {}",name));
                    res = var_data.var_type;
                    pointer_depth = var_data.pointer_depth + 1;
                }

                RpnExpr::Deref(v) => {
                    let name = v.var.value.as_ref().unwrap();
                    let var_data = self.m_vars.get(name).expect(&format!("no var with name: {}",name));
                    res = var_data.var_type;
                    pointer_depth = var_data.pointer_depth - v.stack_depth;
                }

                _ => continue,
            }
        }
        TypeInfo { var_type: res, pointer_depth, }
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

    fn add_var(&mut self,key: String, value: VarData) {
        if self.m_vars.contains_key(&key) {
            panic!("redefinition of variable: {}",key);
        }
        self.m_vars.insert(key, value);
    }

    }
