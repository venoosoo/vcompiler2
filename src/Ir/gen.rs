use std::collections::HashMap;

use crate::Tokenizer::{Token, TokenType};
use crate::Ir::stmt::{Arg, StructArg};

#[derive(Debug)]
pub(crate) struct ExprStack {
    pub(crate) reg: String,
    pub(crate) var_typd: (TokenType, u32),
}
#[derive(Debug)]
// ????????
pub(crate) struct ArrData {
    pub(crate) size: u32,
}
#[derive(Debug)]
pub(crate) struct VarStructData {
    pub(crate) struct_name: String,
}
#[derive(Debug)]
pub(crate) struct VarData {
    pub(crate) stack_pos: i32,
    pub(crate) scope_depth: usize,
    pub(crate) var_type: TokenType,
    pub(crate) arr_data: Option<ArrData>,
    pub(crate) struct_data: Option<VarStructData>,
    pub(crate) pointer_depth: u32,
}


#[derive(Debug, Clone)]
pub(crate) struct FuncData {
    pub(crate) args: Vec<Arg>,
    // return type and pointer depth
    pub(crate) return_type: (Token, u32),
}

#[derive(Debug, Clone)]
pub(crate) struct StructData {
    pub(crate) elements: HashMap<String, StructArg>,
    pub(crate) element_size: u32,
}
