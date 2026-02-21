
use super::*;

use crate::Ir::Stmt;

impl Gen {
    pub fn parse_stmt(&mut self,expr: &mut Stmt) {
        match expr {
                Stmt::CreateVar(v) => {
                    v.eval(self);
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
                    v.eval(self);
                }

                Stmt::ChangePtrValue(v) => {
                    v.eval(self);
                }

                Stmt::InitArray(v) => {
                    v.eval(self);
                }

                Stmt::ChangeVar(v) => {
                    self.eval_expr(&mut v.stmt);
                    let var = self.m_vars.get(&v.var).unwrap();
                    self.emit(format!("    mov {} [rbp - {}], {}",Gen::get_word(var.var_type),var.stack_pos, Gen::get_rax_register(var.var_type)));   
                }
                
                Stmt::IfStmt(v) => {
                    v.eval(self);
                }

                Stmt::WhileStmt(v) => {
                    v.eval(self);
                }
                Stmt::ForStmt(v) => {
                    v.eval(self);
                }
                Stmt::IncVar(v) => {
                    v.eval(self);
                }
                Stmt::DecVar(v) => {
                    v.eval(self);
                }
                Stmt::Ret(v) => {
                    v.eval(self);
                }
                Stmt::InitStruct(v) => {
                    // we already added it earlier while checking for function init
                    // so we just skipping this to not make a copy
                }


                Stmt::ChangeStructValue(v) => {
                    v.eval(self);
                }
                   

                Stmt::CreateStruct(v) => {
                    v.eval(self);
                }

                Stmt::ChangePtrStructValue(v) => {
                    v.eval(self);
                }

                Stmt::InitFunc(v) => {
                    v.eval(self);
                }
                Stmt::ChangeArrElement(v) => {
                    v.eval(self);
                }
                Stmt::FunctionCall(v) => {
                    v.eval(self);
                }
                Stmt::AsmCode(v) => {
                    v.eval(self);
                }
                _ => self::panic!("trying to gen unkown expr: {:?}",expr)
            }
        }
}