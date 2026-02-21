use std::collections::HashMap;

use crate::Tokenizer::{Token, TokenType};
use crate::Ir::expr::RpnExpr;

#[derive(Debug, Clone)]
pub enum Stmt {
    CreateVar(CreateVar),
    OpenScope(OpenScope),
    CloseScope(CloseScope),
    ChangeVar(ChangeVar),
    IfStmt(IfStmt),
    WhileStmt(WhileStmt),
    ForStmt(ForStmt),
    IncVar(IncVar),
    DecVar(DecVar),
    InitFunc(InitFunc),
    Ret(Ret),
    FunctionCall(FunctionCall),
    AsmCode(AsmCode),
    InitArray(InitArray),
    ChangeArrElement(ChangeArrElement),
    CreatePointer(CreatePointer),
    ChangePtrValue(ChangePtrValue),
    InitStruct(InitStruct),
    CreateStruct(CreateStruct),
    ChangeStructValue(ChangeStructValue),
    ChangePtrStructValue(ChangePtrStructValue),
}



#[derive(Debug, Clone)]
pub(crate) struct ChangePtrStructValue {
    pub(crate) struct_name: String,
    pub(crate) value_name: String,
    pub(crate) expr: Vec<RpnExpr>,
}



#[derive(Debug, Clone)]
pub(crate) struct ChangeStructValue {
    pub(crate) struct_name: String,
    pub(crate) value_name: String,
    pub(crate) expr: Vec<RpnExpr>,
}


#[derive(Debug, Clone)]
pub(crate) struct CreateStruct {
    pub(crate) struct_name: String,
    pub(crate) var_name: String,
    pub(crate) pointer_depth: u32,
    pub(crate) expr: Option<Vec<RpnExpr>>,
}

#[derive(Debug, Clone)]
pub(crate) struct StructArg {
    pub(crate) arg_type: Token,
    pub(crate) pointer_depth: u32,
    pub(crate) name: Token,
    pub(crate) pos: u32,
}





#[derive(Debug, Clone)]
pub(crate) struct InitStruct {
    pub(crate) name: String,
    pub(crate) elements: HashMap<String, StructArg>,
}


#[derive(Debug, Clone)]
pub(crate) struct ChangePtrValue {
    pub(crate) var: String,
    pub(crate) stmt: Vec<RpnExpr>,
    pub(crate) pointer_depth: u32
}

#[derive(Debug, Clone)]
pub(crate) struct CreatePointer {
    pub(crate) type_: TokenType,
    pub(crate) var: String,
    pub(crate) stmt: Vec<RpnExpr>,
    pub(crate) pointer_depth: u32
}

#[derive(Debug, Clone)]
pub(crate) struct ChangeArrElement {
    pub(crate) arr_name: Token,
    pub(crate) element: Token,
    pub(crate) expr: Vec<RpnExpr>,
}

#[derive(Debug, Clone)]
pub(crate) struct InitArray {
    pub(crate) name: Token,
    pub(crate) arr_type: Token,
    pub(crate) size: Token,
    pub(crate) data: Vec<Token>,
}



#[derive(Debug, Clone)]
pub(crate) struct AsmCode {
    pub(crate) code: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionCall {
    pub(crate) name: Token,
    pub(crate) args: Vec<Vec<RpnExpr>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Ret {
    pub(crate) expr: Vec<RpnExpr>,
    pub(crate) func_name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct TypeInfo {
    pub(crate) var_type: TokenType,
    pub(crate) pointer_depth: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct InitFunc {
    pub(crate) args: Vec<Arg>,
    pub(crate) name: Token,
    // type and pointer depth
    pub(crate) return_type: TypeInfo,
    pub(crate) data: Vec<Stmt>

}
#[derive(Debug, Clone)]
pub(crate) struct Arg {
    pub(crate) arg_type: Token,
    pub(crate) struct_name: Option<String>,
    pub(crate) pointer_depth: u32,
    pub(crate) name: Token,
}

#[derive(Debug, Clone)]
pub(crate) struct IncVar {
    pub(crate) var: Token,
}

#[derive(Debug, Clone)]
pub(crate) struct DecVar {
    pub(crate) var: Token,
}

#[derive(Debug, Clone)]
pub(crate) struct WhileStmt {
    pub(crate) expr: Vec<RpnExpr>,
    pub(crate) data: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub(crate) struct ForStmt {
    pub(crate) expr1: Box<Stmt>,
    pub(crate) expr2: Vec<RpnExpr>,
    pub(crate) expr3: Box<Stmt>,
    pub(crate) data: Vec<Stmt>,
}


#[derive(Debug, Clone)]
pub(crate) struct IfStmt {
    pub(crate) expr: Vec<RpnExpr>,
    pub(crate) data: Vec<Stmt>,
    pub(crate) else_data: Vec<Stmt>
}
#[derive(Debug, Clone)]
pub(crate) struct CreateVar {
    pub(crate) Type: TokenType,
    pub(crate) var: String,
    pub(crate) stmt: Vec<RpnExpr>,
}
#[derive(Debug, Clone)]
pub(crate) struct ChangeVar {
    pub(crate) var: String,
    pub(crate) stmt: Vec<RpnExpr>,
}

#[derive(Debug, Clone)]
pub(crate) struct OpenScope;
#[derive(Debug, Clone)]
pub(crate) struct CloseScope;