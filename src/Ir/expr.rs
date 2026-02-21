use crate::Tokenizer::Token;


#[derive(Debug, Clone)]
pub(crate) enum RpnExpr {
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
pub(crate) struct GetStructValue {
    pub(crate) var_name: String,
    pub(crate) struct_value_name: String,
}


#[derive(Debug, Clone)]
pub(crate) struct GetSizeOf {
    pub(crate) var: Token,
}


#[derive(Debug, Clone)]
pub(crate) struct GetAddr {
    pub(crate) var: Token,
}

#[derive(Debug, Clone)]
pub(crate) struct Deref {
    pub(crate) var: Token,
    pub(crate) stack_depth: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct GetArrayValue {
    pub(crate) name: Token,
    pub(crate) index: Token,
}

#[derive(Debug, Clone)]
pub(crate) struct Negative {
    pub(crate) data: Token,
}
#[derive(Debug, Clone)]
pub(crate) struct PushNum {
    pub(crate) data: Token,
}
#[derive(Debug, Clone)]
pub(crate) struct PushVar {
    pub(crate) data: Token,
}
#[derive(Debug, Clone)]
pub(crate) struct Operator {
    pub(crate) data: Token,
}
#[derive(Debug, Clone)]
pub(crate) struct Function {
    pub(crate) name: Token,
    pub(crate) args: Vec<Token>,
}