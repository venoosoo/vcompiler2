//! Memory-related expression evaluation.
//!
//! This module implements code generation for expression nodes that interact
//! with memory or stack storage. Unlike pure arithmetic expressions, the
//! expressions defined here:
//!
//! - Access local variables from the current stack frame
//! - Compute addresses (`&var`)
//! - Perform pointer dereferencing (`*ptr`)
//! - Evaluate `sizeof`-like operations
//! - Push immediate numeric literals
//!
//! These implementations emit x86-64 assembly instructions through [`Gen`]
//! and use [`ExprStackHelper`] to determine the correct destination register
//! or stack slot for intermediate expression values.
//!
//! # Responsibilities
//!
//! - Load values from `[rbp - offset]`
//! - Compute addresses using `lea`
//! - Follow pointer chains during dereference
//! - Handle pointer depth tracking
//! - Push evaluated values onto the expression stack
//!
//! All nodes here are part of the backend lowering phase, where IR-level
//! expressions are translated directly into assembly instructions.
//! 
//! 

use crate::Ir::{expr::{Deref, GetAddr, GetArrayValue, GetSizeOf, GetStructValue, Negative, Operator, PushNum, PushVar}, r#gen};
use super::*;
impl PushNum {
    /// Evaluates an integer literal expression.
    ///
    /// Emits a `mov` instruction loading the immediate value into the
    /// appropriate register determined by [`ExprStackHelper`], and pushes
    /// the resulting value onto the expression stack.
    pub fn eval(&self, stack_helper: &mut ExprStackHelper, gen_help: &mut Gen){
        let val = self.data.value.as_ref().unwrap();
        let reg = stack_helper.get_reg(TokenType::IntType, 0);
        gen_help.emit(format!("    mov {}, {}",reg, val));
        stack_helper.push(ExprStack {reg, var_type: TokenType::IntType, pointer_depth: 0});
    }
}

impl GetSizeOf {
    /// Evaluates a `sizeof`-like expression.
    ///
    /// Determines the size of a variable's type using [`Gen::get_size`].
    /// If the variable has pointer depth greater than zero, the result is
    /// treated as an 8-byte pointer size.
    ///
    /// The computed size is loaded into a register and pushed onto the
    /// expression stack as an integer value.
    pub fn eval(&self, stack_helper: &mut ExprStackHelper, gen_help: &mut Gen) {
        let name = self.var.value.as_ref().unwrap();
        let (var_type, pointer_depth) = {
            let var_data = gen_help.m_vars.get(name)
                .expect(&format!("no var with name: {}", name));
            (var_data.var_type, var_data.pointer_depth)
        };
        let mut size = gen_help.get_size(var_type);
        let reg = stack_helper.get_reg(var_type, pointer_depth);
        if pointer_depth > 0 {
            size = 8;
        }
        gen_help.emit(format!("    mov {}, {}",reg, size));
        stack_helper.push(ExprStack { reg, var_type, pointer_depth: pointer_depth});
    }
}

impl Deref {
    /// Evaluates a pointer dereference expression.
    ///
    /// This implementation:
    /// 1. Loads the base pointer value from the stack frame.
    /// 2. Iteratively dereferences memory based on `stack_depth`.
    /// 3. Alternates between temporary register usage to follow pointer chains.
    /// 4. Produces the final dereferenced value in the target register.
    ///
    /// Pointer depth and type metadata are preserved and pushed onto
    /// the expression stack.
    pub fn eval(&self, stack_helper: &mut ExprStackHelper, gen_help: &mut Gen) {
        let name = self.var.value.as_ref().unwrap();
        let (var_type , pointer_depth,stack_pos) = {
            let var_data = gen_help.m_vars.get(name)
            .expect(&format!("no var with name: {}",name));
        (var_data.var_type, var_data.pointer_depth, var_data.stack_pos)
        };
        let reg = stack_helper.get_reg(var_type, pointer_depth);
        gen_help.emit(format!("    mov rsi, [rbp - {}]",stack_pos));
        for i in 0..self.stack_depth {
            if i % 2 == 0 {
                gen_help.emit(format!("    mov {}, [rsi]", reg));
            } else {
                gen_help.emit(format!("    mov rsi, [{}]",reg));
            }
        }
        if self.stack_depth % 2 == 0 {
            gen_help.emit(format!("    mov {}, rsi", reg));
        }
        stack_helper.push(ExprStack { reg: reg, var_type, pointer_depth,});
    }
}

impl GetAddr {
    /// Evaluates an address-of expression (`&var`).
    ///
    /// Computes the stack address of a local variable using `lea` and
    /// stores it as a pointer value in the destination register.
    ///
    /// The resulting expression increases pointer depth by one and
    /// is pushed onto the expression stack.
    pub fn eval(&self, stack_helper: &mut ExprStackHelper, gen_help: &mut Gen) {

        let name = self.var.value.as_ref().unwrap();
        // the pointer is always 8 bytes
        
        let (var_type, pointer_depth, stack_pos) = {
            let var_data = gen_help.m_vars.get(name)
            .expect(&format!("no var with name: {}",name));
        (var_data.var_type, var_data.pointer_depth, var_data.stack_pos)
    };
    
        let reg = stack_helper.get_reg(TokenType::LongType, 1);

        gen_help.emit(format!("    lea rsi, [rbp-{}]",stack_pos));
        gen_help.emit(format!("    mov {}, rsi",reg));
        stack_helper.push(ExprStack { reg, var_type, pointer_depth: pointer_depth + 1 })
    
    }
}

impl PushVar {
    /// Evaluates a variable access expression.
    ///
    /// Loads the value of a local variable from the stack frame.
    /// If the variable represents an array, its base address is loaded
    /// using `lea` instead of dereferencing the value.
    ///
    /// The loaded value (or address) is pushed onto the expression stack
    /// with its associated type and pointer depth.
    pub fn eval(&self, stack_helper: &mut ExprStackHelper, gen_help: &mut Gen) {
        let name = self.data.value.as_ref().unwrap();
        let var: &VarData = gen_help.m_vars.get(name)
            .expect(&format!("unkown var: {}",name));

        let (var_type, pointer_depth, stack_pos) = {
            let var_data = gen_help.m_vars.get(name)
            .expect(&format!("no var with name: {}",name));
            (var_data.var_type, var_data.pointer_depth, var_data.stack_pos)
        };

        let reg = stack_helper.get_reg(var_type, pointer_depth);

        if var.arr_data.is_some() {
            gen_help.emit(format!("    lea  {}, [rbp - {}]", reg, stack_pos));
        }
        else {
            gen_help.emit(format!("    mov {}, [rbp - {}]",reg,stack_pos));
        }
        stack_helper.push(ExprStack { reg, var_type, pointer_depth });
    }
}


impl GetStructValue {
    pub fn eval(&self, stack_helper: &mut ExprStackHelper, gen_help: &mut Gen) {
        let var_data = gen_help.m_vars.get(&self.var_name)
        .expect(&format!("no var with name: {}",self.var_name));
        
        let stack_pos = var_data.stack_pos;
        if let Some(val) = var_data.struct_data.as_ref() {
            let struct_data = gen_help.structs.get(&val.struct_name)
            .expect(&format!("no struct with name: {:?}",val.struct_name));
            
            let element_size = struct_data.element_size;
            let (arg_type, pointer_depth, pos) = {
                let res = struct_data.elements.get(&self.struct_value_name)
                .expect(&format!("in var: {:?} there's no field: {:?}",self.var_name, self.struct_value_name));
                (res.arg_type.clone(), res.pointer_depth, res.pos)
            };

            let reg = stack_helper.get_reg(arg_type.token, pointer_depth);
            let value = stack_pos - (pos as i32 * element_size as i32);
            gen_help.emit(format!("    mov {}, [rbp - {}]",reg,value));
            
            stack_helper.push(ExprStack { reg, var_type: arg_type.token, pointer_depth: pointer_depth, })
        }
            
    }
}


impl GetArrayValue {
    pub fn eval(&self, stack_helper: &mut ExprStackHelper, gen_help: &mut Gen) {
        let name = self.name.value.as_ref().unwrap();
        let (stack_pos, pointer_depth,var_type) = {

            let var_data = gen_help.m_vars.get(name)
            .expect(&format!("no var with name: {}",name));
            (var_data.stack_pos, var_data.pointer_depth,var_data.var_type)
        };
        let reg = stack_helper.get_reg(var_type, pointer_depth);
        if self.index.token == TokenType::Var {
            let index_name = self.name.value.as_ref().unwrap();
            let (index_stack_pos, index_type) = {
                
                let index_data = gen_help.m_vars.get(index_name)
                .expect(&format!("no var with name: {}",index_name));
                (index_data.stack_pos, index_data.var_type)
            };
            let rsi_reg = Gen::get_rsi_regsiter(index_type);

            gen_help.emit(format!("    mov {}, [rbp - {}]",rsi_reg,index_stack_pos));
            gen_help.emit(format!("    mov {}, [rbp - {}]",reg,stack_pos));
            gen_help.emit(format!("    add {}, {}",reg, rsi_reg));
            gen_help.emit(format!("    mov {}, [{}]",reg,reg));
            
        }
        else {
            let index_value = self.index.value.as_ref().unwrap().parse::<u32>().unwrap();
            let element_stack_pos = stack_pos as u32 - (index_value * gen_help.get_size(var_type));
            gen_help.emit(format!("    mov {}, [rbp - {}]",reg,element_stack_pos));
        }
    }
}



impl Negative {
    pub fn eval(&mut self,stack_helper: &mut ExprStackHelper, gen_help: &mut Gen) {
        if self.data.token == TokenType::Var {
            let name = self.data.value.clone().unwrap();
            let var = gen_help.m_vars.get(&name).expect(format!("unkown var: {}",&name).as_str());
            let arg = stack_helper.get_reg(var.var_type, var.pointer_depth);
            let push_var = PushVar {
                data: self.data.clone(),
            };
            push_var.eval(stack_helper, gen_help);
            gen_help.emit(format!("    neg {}",arg));
        }
        else if self.data.token == TokenType::Num {
            self.data.value = Some(format!("-{}",self.data.value.clone().unwrap()));
            let push_num = PushNum {
                data: self.data.clone()
            };
            push_num.eval(stack_helper, gen_help);
        }
}
}


impl Operator {
    pub fn eval(&mut self,stack_helper: &mut ExprStackHelper, gen_help: &mut Gen) {
        let t = &self.data.token;
    
        match t {
            // ===== binary ops =====
            TokenType::Add
            | TokenType::Sub
            | TokenType::Mul
            | TokenType::Div
            | TokenType::Remainder => {
                let rhs = stack_helper.pop().expect("rhs missing");
                let lhs = stack_helper.pop().expect("lhs missing");
                let res = gen_help.compare_reg(&lhs.reg, &rhs.reg);

                let res_reg = {
                    if Gen::is_num(lhs.var_type) {
                        Gen::get_rdx_register(lhs.var_type)
                    }
                    else {
                        Gen::get_rdx_register(rhs.var_type)
                    }
                };

                if *t == TokenType::Add {
                    if lhs.pointer_depth > 0 {
                        gen_help.emit(format!("    imul {}, {}",res.1, gen_help.get_size(lhs.var_type)));
                    }
                    else if rhs.pointer_depth > 0 {
                        gen_help.emit(format!("    imul {}, {}",res.0, gen_help.get_size(rhs.var_type)));
                    }
                }

                if *t == TokenType::Sub {
                    if lhs.pointer_depth > 0 {
                        gen_help.emit(format!("    imul {}, {}",res.1, gen_help.get_size(lhs.var_type)));
                    }
                }
                
                match t {
                    TokenType::Add => gen_help.emit(format!("    add {}, {}", res.0, res.1)),
                    TokenType::Sub => gen_help.emit(format!("    sub {}, {}", res.0, res.1)),
                    TokenType::Mul => gen_help.emit(format!("    imul {}, {}", res.0, res.1)),
                    TokenType::Div => {
                        gen_help.emit("    cdq".into());
                        gen_help.emit(format!("    idiv {}", res.1));
                    }
                    TokenType::Remainder => {
                        gen_help.emit(format!("    cqo"));
                        gen_help.emit(format!("    idiv {}",res.1));
                        gen_help.emit(format!("    mov {}, {}",res.0, res_reg));
                    }
                    _ => unreachable!(),
                }
                stack_helper.push(ExprStack { reg: format!("{}",lhs.reg), var_type: TokenType::Num, pointer_depth: 0 });
            }
            
            // ===== comparisons =====
            TokenType::AsertEq
            | TokenType::NotEq
            | TokenType::Less
            | TokenType::LessThan
            | TokenType::More
            | TokenType::MoreThan => {
                let rhs = stack_helper.pop().unwrap();
                let lhs = stack_helper.pop().unwrap();
                
                let res = gen_help.compare_reg(&lhs.reg, &rhs.reg);
                
                gen_help.emit(format!("    cmp {}, {}",res.0, res.1));
                
                let set = match t {
                    TokenType::AsertEq  => "sete",
                    TokenType::NotEq    => "setne",
                    TokenType::Less     => "setl",
                    TokenType::LessThan => "setle",
                    TokenType::More     => "setg",
                    TokenType::MoreThan => "setge",
                    _ => unreachable!(),
                };
                
                gen_help.emit(format!("    {} al", set));
                gen_help.emit("    movzx rax, al".into());
                
                stack_helper.push(ExprStack { reg: "rax".into(), var_type: TokenType::Num, pointer_depth: 0 });
            }
            
            _ => self::panic!("unsupported operator {:?}", t),
        }
    }
}