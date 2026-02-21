
use crate::Ir::expr::RpnExpr;

use super::*;
use crate::Tokenizer::{TokenType};
use crate::Ir::r#gen::ExprStackHelper;


impl ExprStackHelper {
    pub fn get_reg(&self, token_type: TokenType,pointer_depth: u32) -> String {
        if self.stack.is_empty() {
            if pointer_depth > 0 {return "rax".to_string()}
            Gen::get_rax_register(token_type)
        }
        else if self.stack.len() == 1 {
            if pointer_depth > 0 {return "rbx".to_string()}
            Gen::get_rbx_register(token_type)
        }
        else {
            let slot = Gen::calc_expr_stack_size(&self.stack) + 8;
            format!("    [rbp - {}]",slot)
        }
    }

    pub fn push(&mut self, value: ExprStack) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<ExprStack> {
        self.stack.pop()
    }

}



impl Gen {
    pub fn eval_expr(&mut self,  rpn: &mut Vec<RpnExpr>) {
        let mut stack_helper = ExprStackHelper {
            stack: Vec::new(),
        };
        for expr in rpn.iter_mut() {
            match expr {
                RpnExpr::PushNum(v) => {
                    v.eval(&mut stack_helper, self);
                }

                RpnExpr::GetStructValue(v) => {
                    v.eval(&mut stack_helper, self);
                        
                }

                RpnExpr::GetSizeOf(v) => {
                    v.eval(&mut stack_helper, self);
                }

                RpnExpr::Deref(v) => {
                    v.eval(&mut stack_helper, self);
                }


                RpnExpr::GetAddr(v) => {
                    v.eval(&mut stack_helper, self);
                }

                RpnExpr::GetArrayValue(v) => {                
                    v.eval(&mut stack_helper, self);
                }

                RpnExpr::Negative(v) => {
                    v.eval(&mut stack_helper, self);
                }

                
                RpnExpr::PushVar( v) => {
                    v.eval(&mut stack_helper, self);
                }
                
                RpnExpr::Operator(v) => {
                    v.eval(&mut stack_helper, self);
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