use crate::Tokenizer::Token;


#[derive(Debug, Clone)]
pub enum RpnExpr {
    PushNum(PushNum),
    PushVar(PushVar),
    Operator(Operator),
    Function(Function),
    Negative(Negative),
    GetArrayValue(GetArrayValue),
    Deref(Deref),
    GetAddr(GetAddr),
    GetSizeOf(GetSizeOf),
    GetStructValue(GetStructValue),
}


#[derive(Debug, Clone)]
pub struct GetStructValue {
    pub var_name: String,
    pub struct_value_name: String,
}


#[derive(Debug, Clone)]
pub struct GetSizeOf {
    pub var: Token,
}


#[derive(Debug, Clone)]
pub struct GetAddr {
    pub var: Token,
}

#[derive(Debug, Clone)]
pub struct Deref {
    pub var: Token,
    pub stack_depth: u32,
}

#[derive(Debug, Clone)]
pub struct GetArrayValue {
    pub name: Token,
    pub index: Token,
}

#[derive(Debug, Clone)]
pub struct Negative {
    pub data: Token,
}
#[derive(Debug, Clone)]
pub struct PushNum {
    pub data: Token,
}
#[derive(Debug, Clone)]
pub struct PushVar {
    pub data: Token,
}
#[derive(Debug, Clone)]
pub struct Operator {
    pub data: Token,
}
#[derive(Debug, Clone)]
pub struct Function {
    pub name: Token,
    pub args: Vec<Token>,
}