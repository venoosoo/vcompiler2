use std::collections::HashMap;

use crate::Tokenizer::{Token, TokenType};
use crate::Ir::expr::RpnExpr;

#[derive(Debug, Clone)]
pub enum Stmt {
    Var(CreateVar),
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
pub struct ChangePtrStructValue {
    pub struct_name: String,
    pub value_name: String,
    pub expr: Vec<RpnExpr>,
}



#[derive(Debug, Clone)]
pub struct ChangeStructValue {
    pub struct_name: String,
    pub value_name: String,
    pub expr: Vec<RpnExpr>,
}


#[derive(Debug, Clone)]
pub struct CreateStruct {
    pub struct_name: String,
    pub var_name: String,
    pub pointer_depth: u32,
    pub expr: Option<Vec<RpnExpr>>,
}

#[derive(Debug, Clone)]
pub struct StructArg {
    pub arg_type: Token,
    pub pointer_depth: u32,
    pub name: Token,
    pub pos: u32,
}





#[derive(Debug, Clone)]
pub struct InitStruct {
    pub name: String,
    pub elements: HashMap<String, StructArg>,
}


#[derive(Debug, Clone)]
pub struct ChangePtrValue {
    pub var: String,
    pub stmt: Vec<RpnExpr>,
    pub pointer_depth: u32
}

#[derive(Debug, Clone)]
pub struct CreatePointer {
    pub type_: TokenType,
    pub var: String,
    pub stmt: Vec<RpnExpr>,
    pub pointer_depth: u32
}

#[derive(Debug, Clone)]
pub struct ChangeArrElement {
    pub arr_name: Token,
    pub element: Token,
    pub expr: Vec<RpnExpr>,
}

#[derive(Debug, Clone)]
pub struct InitArray {
    pub name: Token,
    pub arr_type: Token,
    pub size: Token,
    pub data: Vec<Token>,
}



#[derive(Debug, Clone)]
pub struct AsmCode {
    pub code: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: Token,
    pub args: Vec<Vec<RpnExpr>>,
}

#[derive(Debug, Clone)]
pub struct Ret {
    pub expr: Vec<RpnExpr>,
    pub func_name: String,
}

#[derive(Debug, Clone)]
pub struct InitFunc {
    pub args: Vec<Arg>,
    pub name: Token,
    // type and pointer depth
    pub return_type: (Token, u32),
    pub data: Vec<Stmt>

}
#[derive(Debug, Clone)]
pub struct Arg {
    pub arg_type: Token,
    pub struct_name: Option<String>,
    pub pointer_depth: u32,
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct IncVar {
    pub var: Token,
}

#[derive(Debug, Clone)]
pub struct DecVar {
    pub var: Token,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub expr: Vec<RpnExpr>,
    pub data: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub expr1: Box<Stmt>,
    pub expr2: Vec<RpnExpr>,
    pub expr3: Box<Stmt>,
    pub data: Vec<Stmt>,
}


#[derive(Debug, Clone)]
pub struct IfStmt {
    pub expr: Vec<RpnExpr>,
    pub data: Vec<Stmt>,
    pub else_data: Vec<Stmt>
}
#[derive(Debug, Clone)]
pub struct CreateVar {
    pub Type: TokenType,
    pub var: String,
    pub stmt: Vec<RpnExpr>,
}
#[derive(Debug, Clone)]
pub struct ChangeVar {
    pub var: String,
    pub stmt: Vec<RpnExpr>,
}

#[derive(Debug, Clone)]
pub struct OpenScope;
#[derive(Debug, Clone)]
pub struct CloseScope;