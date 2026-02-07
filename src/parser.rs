
use crate::tokenizer::Token;
use crate::tokenizer::TokenType;


pub struct Parser {
    m_tokens: Vec<Token>,
    m_index: usize,
    expressions: Vec<Expr>,
    func_name: String,
}



pub struct Program(Vec<Expr>);

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, expr) in self.0.iter().enumerate() {
            writeln!(f, "[{}] {:?}", i, expr)?; 
        }
        Ok(())
    }
}


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


#[derive(Debug, Clone)]
pub enum Expr {
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
}

#[derive(Debug, Clone)]
pub struct ChangePtrValue {
    pub Var: String,
    pub Expr: Vec<RpnExpr>,
    pub pointer_depth: u32
}

#[derive(Debug, Clone)]
pub struct CreatePointer {
    pub Type: TokenType,
    pub Var: String,
    pub Expr: Vec<RpnExpr>,
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
    pub data: Vec<Expr>

}
#[derive(Debug, Clone)]
pub struct Arg {
    pub arg_type: Token,
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
    pub data: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub expr1: Box<Expr>,
    pub expr2: Vec<RpnExpr>,
    pub expr3: Box<Expr>,
    pub data: Vec<Expr>,
}


#[derive(Debug, Clone)]
pub struct IfStmt {
    pub expr: Vec<RpnExpr>,
    pub data: Vec<Expr>,
    pub else_data: Vec<Expr>
}
#[derive(Debug, Clone)]
pub struct CreateVar {
    pub Type: TokenType,
    pub Var: String,
    pub Expr: Vec<RpnExpr>,
}
#[derive(Debug, Clone)]
pub struct ChangeVar {
    pub var: String,
    pub Expr: Vec<RpnExpr>,
}

#[derive(Debug, Clone)]
pub struct OpenScope {}
#[derive(Debug, Clone)]
pub struct CloseScope {}


impl Parser {
    pub fn new(m_tokens: Vec<Token>) -> Self {
        Parser {
            m_tokens,
            m_index: 0,
            expressions: Vec::new(),
            func_name: String::new(),
        }
    }

    fn peek(&self, offset: usize) -> &Token {
        let pos: usize = self.m_index + offset;
        if self.m_index >= self.m_tokens.len() {
            panic!("Trying to parse token more than token array has\nm_index: {}",self.m_tokens.len());
        }
        &self.m_tokens[pos]
    }


    fn parse_func(&mut self, var_token: Token, type_token: (Token, u32)) -> Option<Expr> {
        self.consume();
        let mut args: Vec<Arg> = Vec::new();
        let mut pointer_depth = 0;
        while self.peek(0).token != TokenType::CloseParen {
            let arg_type = self.consume();
            while self.peek(0).token == TokenType::Mul {
                pointer_depth += 1;
                self.consume();
            }
            let arg_name = self.consume();
            if self.peek(0).token == TokenType::Coma {
                self.consume();
            }
            let arg = Arg {
                arg_type,
                pointer_depth,
                name: arg_name,
            };
            args.push(arg);
        
        }
        self.consume();
        let mut expr_arr: Vec<Expr> = Vec::new();
        let mut depth = 0;
        self.func_name = var_token.value.clone().unwrap();
        while self.peek(0).token != TokenType::CloseScope || depth > 0 {
            let expr: Expr = self.parse_expr().unwrap();
            match &expr {
                Expr::ForStmt(_) => depth += 1,
                Expr::WhileStmt(_) => depth += 1,
                Expr::CloseScope(_) => depth -=1,
                _ => depth += 0,
            }
            expr_arr.push(expr);
        }
                
        let init_func = InitFunc {
            name: var_token,
            return_type: type_token,
            args,
            data: expr_arr
        };

        return Some(Expr::InitFunc(init_func));
    }
    

    fn consume(&mut self) -> Token {
        if self.m_index >= self.m_tokens.len() {
            panic!("Trying to consume more than m_src len");
        }
        self.m_tokens.remove(0)
    }

    // add proper error handling
    fn eval_expr(&mut self) -> Vec<RpnExpr> {
        let mut output: Vec<RpnExpr> = Vec::new();
        let mut op_stack: Vec<Token> = Vec::new();
        let mut previous_token: Option<Token> = None;
        while !matches!(
            self.peek(0).token,
            TokenType::Semi | TokenType::OpenScope | TokenType::Coma
        ) {
            if self.peek(0).token == TokenType::CloseParen && self.peek(1).token == TokenType::Semi {
                break;
            }
            let token = self.consume();
            let token_copy = token.clone();

            match token.token {
                TokenType::Num | TokenType::CharValue => {
                    output.push(RpnExpr::PushNum(PushNum { data: token }));
                }


                TokenType::Var => {
                    if self.peek(0).token == TokenType::OpenParen {
                        let func = self.parse_rpn_function(token);
                        output.push(func);
                    } else if self.peek(0).token == TokenType::OpenBracket {
                        self.consume();
                        //redo this to take expr
                        let index = self.consume();
                        self.consume();
                        let get_array_value = GetArrayValue {
                            name: token,
                            index,
                        };
                        output.push(RpnExpr::GetArrayValue(get_array_value));
                    } else {
                        output.push(RpnExpr::PushVar(PushVar { data: token }));
                    }
                }


                TokenType::Address => {
                        let var = self.consume();
                        let res = GetAddr {
                            var,
                        };  
                        output.push(RpnExpr::GetAddr(res));
                        continue;
                }


                TokenType::OpenParen => {
                    op_stack.push(token);
                }

                TokenType::CloseParen => {
                    while let Some(op) = op_stack.last() {
                        if op.token == TokenType::OpenParen {
                            break;
                        }
                        let op = op_stack.pop().unwrap();
                        output.push(RpnExpr::Operator(Operator { data: op }));
                    }
                    op_stack.pop(); // pop '('
                }

                _ if Parser::is_operator(&token) => {
                    if previous_token.is_some() {

                        if Parser::is_operator(&previous_token.clone().unwrap()) && token.token == TokenType::Mul {
                            let mut stack_depth = 1;
                            while self.peek(0).token == TokenType::Mul {
                                stack_depth += 1;
                                self.consume();
                            }
                            let var = self.consume();
                            let res = Deref {
                                var,
                                stack_depth,
                            };
                            output.push(RpnExpr::Deref(res));
                            continue;
                        }

                        if Parser::is_operator(&previous_token.clone().unwrap()) && token.token == TokenType::Sub {
                            output.push(RpnExpr::Negative(Negative { data: self.consume() }));
                            continue;
                        }
                    }
                    else {
                        if token.token == TokenType::Mul {
                            let mut stack_depth = 1;
                            while self.peek(0).token == TokenType::Mul {
                                stack_depth += 1;
                                self.consume();
                            }
                            let var = self.consume();
                            let res = Deref {
                                var,
                                stack_depth,
                            };
                            output.push(RpnExpr::Deref(res));
                            continue;
                        }

                        if token.token == TokenType::Sub {
                            output.push(RpnExpr::Negative(Negative { data: self.consume() }));
                            continue;
                        }
                    }
                    while let Some(top) = op_stack.last() {
                        if top.token == TokenType::OpenParen {
                            break;
                        }

                        if Parser::bigger_operator(&token)
                            <= Parser::bigger_operator(top)
                        {
                            let op = op_stack.pop().unwrap();
                            output.push(RpnExpr::Operator(Operator { data: op }));
                        } else {
                            break;
                        }
                    }
                    op_stack.push(token);
                }

                _ => {}
            }
            previous_token = Some(token_copy);
        }

        while let Some(op) = op_stack.pop() {
            output.push(RpnExpr::Operator(Operator { data: op }));
        }


        output
    }


    fn parse_rpn_function(&mut self, name: Token) -> RpnExpr {
        // consume '('
        self.consume();

        let mut args = Vec::new();

        while self.peek(0).token != TokenType::CloseParen {
            let tok = self.consume();

            if tok.token != TokenType::Coma {
                args.push(tok);
            }
        }

        // consume ')'
        self.consume();

        RpnExpr::Function(Function {
            name,
            args,
        })
    }

    fn bigger_operator(token: &Token) -> i32 {
        match token.token {
            TokenType::Add => 1,
            TokenType::Sub => 1,
            TokenType::Mul => 2,
            TokenType::Div => 2,
            TokenType::And => 1,
            TokenType::Or => 1,
            TokenType::AsertEq => 1,
            TokenType::Not => 1,
            TokenType::NotEq => 1,
            TokenType::Less => 1,
            TokenType::LessThan => 1,
            TokenType::More => 1,
            TokenType::Remainder => 2,
            TokenType::MoreThan => 1,
            _ => panic!("Starnge token in bigger_operator"),
        }
    }

    fn is_operator(token: &Token) -> bool {
        match  token.token {
            TokenType::Add => true,
            TokenType::Sub => true,
            TokenType::Mul => true,
            TokenType::Div => true,
            TokenType::And => true,
            TokenType::Or => true,
            TokenType::AsertEq => true,
            TokenType::Not => true,
            TokenType::NotEq => true,
            TokenType::Less => true,
            TokenType::LessThan => true,
            TokenType::More => true,
            TokenType::Remainder => true,
            TokenType::MoreThan => true,
            _ => false,
        }
    }

    fn is_type(token: &Token) -> bool {
        match token.token {
            TokenType::IntType => true,
            TokenType::CharType => true,
            TokenType::LongType => true,
            TokenType::ShortType => true,
            TokenType::Void => true,
            _ => false,
        }
    }

    pub fn parse(&mut self) -> Vec<Expr> {
        while !self.m_tokens.is_empty() {
            if let Some(expr) = self.parse_expr() {
                self.expressions.push(expr);
            } else {
                panic!("Unexpected token: {:?} at {}", self.peek(0).token, self.m_index);
            }
        }

        self.expressions.clone()
    }

    fn gen_init_func(&mut self,var: Token) -> FunctionCall {
        if self.peek(0).token != TokenType::OpenParen {
            panic!("Expected '('");
        }
        self.consume();
        let mut args: Vec<Vec<RpnExpr>> = Vec::new();
        while self.peek(0).token != TokenType::CloseParen {
            let expr = self.eval_expr();
            if self.peek(0).token == TokenType::Coma {
                self.consume();
            }
            args.push(expr);
        }
        if self.peek(0).token != TokenType::CloseParen {
            panic!("Expected ')'");
        }
        self.consume();
        let func_call = FunctionCall {
            name: var,
            args,
        };
        func_call
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        if Parser::is_type(self.peek(0)) {
            let type_token = self.consume(); 
            let var_token  = self.consume();

            // pointer
            if var_token.token == TokenType::Mul {
                let mut stack_depth: u32 = 1;
                while self.peek(0).token == TokenType::Mul {
                    stack_depth += 1;
                    self.consume();
                }
                let var_name = self.consume();
                if self.peek(0).token == TokenType::OpenParen {
                    return self.parse_func(var_name, (type_token, stack_depth));
                }

                if self.peek(0).token == TokenType::Eq {
                    let expr = self.eval_expr();
                    if self.peek(0).token != TokenType::Semi {
                        panic!("excpected semi colon");
                    }
                    self.consume();
                    let res = CreatePointer {
                        Type: type_token.token,
                        Var: var_name.value.unwrap(),
                        Expr: expr,
                        pointer_depth: stack_depth,
                    };
                    return Some(Expr::CreatePointer(res))

                } else if self.peek(0).token == TokenType::Semi {
                    self.consume();
                    let mut res: Vec<RpnExpr> = Vec::new();
                    let expr = RpnExpr::PushNum(PushNum { data: Token { token: TokenType::Num, value: Some("0xDEADBEEFDEADBEEF".to_string()) } });
                    res.push(expr);
                    let some =  CreatePointer { 
                        Type:TokenType::Num, 
                        Var: var_name.value.unwrap(), 
                        Expr: res,
                        pointer_depth: stack_depth,
                    };
                    return Some(Expr::CreatePointer(some));

                }
            }

            //init array 
            if self.peek(0).token == TokenType::OpenBracket {
                self.consume();
                let arr_size = self.consume();
                self.consume();
                let mut data: Vec<Token> = Vec::new();
                if self.peek(0).token == TokenType::Eq {
                    self.consume();
                    self.consume();
                    while self.peek(0).token != TokenType::CloseScope {
                        let num = self.consume();
                        if self.peek(0).token == TokenType::Coma {
                            self.consume();
                        }
                        data.push(num);

                    }
                    self.consume();
                }
                let init_array = InitArray {
                    name: var_token,
                    arr_type: type_token,
                    size: arr_size,
                    data: data,
                };

                if self.peek(0).token != TokenType::Semi {
                    panic!("Expected semi colon");
                }
                self.consume();

                return Some(Expr::InitArray(init_array));
            }

            if self.peek(0).token == TokenType::Semi {
                self.consume();
                let some =  PushNum { data: Token { token: TokenType::Num, value: Some("0".to_string()) } };
                let expr = RpnExpr::PushNum(some);
                let mut res: Vec<RpnExpr> = Vec::new();
                res.push(expr);

                let new_var = CreateVar {
                    Type: type_token.token,
                    Var: var_token.value.clone().unwrap(),
                    Expr: res,
                };
                
                return Some(Expr::Var(new_var));
            }
            // create var
            if self.peek(0).token == TokenType::Eq {
                self.consume(); // Consume '='


                let res: Vec<RpnExpr> = self.eval_expr();
                
                if self.peek(0).token != TokenType::Semi {
                    panic!("Expected semi colon");
                }
                self.consume();
                let new_var = CreateVar {
                    Type: type_token.token,
                    Var: var_token.value.clone().unwrap(),
                    Expr: res,
                };
                
                return Some(Expr::Var(new_var));
            }
            // init function
            else if self.peek(0).token == TokenType::OpenParen {
                // the pointer depth will always be zero because if we had * in return type
                // it would be in another section
                return self.parse_func(var_token, (type_token, 0));

            }
        }

        if self.peek(0).token == TokenType::Mul {
            self.consume();
            let mut pointer_depth: u32 = 1;
            while self.peek(0).token == TokenType::Mul {
                self.consume();
                pointer_depth += 1;
            }
            let var = self.consume();
            if self.peek(0).token == TokenType::Eq {
                self.consume();
                let expr = self.eval_expr();
                if self.peek(0).token != TokenType::Semi {
                    panic!("no semi colon");
                }
                self.consume();
                let res = ChangePtrValue {
                    Var: var.value.unwrap(),
                    Expr: expr,
                    pointer_depth,
                };
                return Some(Expr::ChangePtrValue(res));

            } else {
                panic!("strange syntax pointer");
            }
        }

        if self.peek(0).token == TokenType::OpenScope {
            self.consume();
            let res = OpenScope { };
            return Some(Expr::OpenScope(res));
        }
        if self.peek(0).token == TokenType::CloseScope {
            self.consume();
            let res = CloseScope {  };
            return Some(Expr::CloseScope(res));
        }
        if self.peek(0).token == TokenType::Var {
            let var = self.consume();
            if var.value.as_deref() == Some("asm") {
                let mut asm_code: Vec<String> = Vec::new();
                self.consume();
                while self.peek(0).token != TokenType::CloseScope {
                    let str = self.consume();
                    asm_code.push(str.value.unwrap());
                }
                self.consume();
                let res = AsmCode {
                    code: asm_code,
                };
                return Some(Expr::AsmCode(res))
            }
            if var.value.as_deref() == Some("sizeof") {
                self.consume();
                let var = self.consume();
                self.consume();
                if self.peek(0).token != TokenType::Semi {
                        panic!("Expected semi colon");
                    }
                    self.consume();

            }
            // change array element
            if self.peek(0).token == TokenType::OpenBracket {
                self.consume();
                let element = self.consume();
                self.consume();
                if self.peek(0).token == TokenType::Eq {
                    self.consume();
                    let res = self.eval_expr();
                    if self.peek(0).token != TokenType::Semi {
                        panic!("Expected semi colon");
                    }
                    self.consume();
                    let change_arr_elemnet = ChangeArrElement {
                        arr_name: var,
                        element,
                        expr: res,
                    };
                    return Some(Expr::ChangeArrElement(change_arr_elemnet));
                }
            }


            if self.peek(0).token == TokenType::Eq {
                self.consume();
                let res = self.eval_expr();
                if self.peek(0).token != TokenType::Semi && self.peek(0).token != TokenType::CloseParen {
                    panic!("Expected semi colon or ')'");
                }
                self.consume();
                let change_var = ChangeVar {
                    Expr: res,
                    var: var.value.unwrap(),
                };
                return Some(Expr::ChangeVar(change_var));
            }
            if self.peek(0).token == TokenType::Inc {
                self.consume();
                if self.peek(0).token != TokenType::CloseParen {
                    if self.peek(0).token != TokenType::Semi {
                        println!("excpected ;");
                    }
                    else {
                        self.consume();
                    }
                }
                let inc_var = IncVar {
                    var
                };
                return  Some(Expr::IncVar(inc_var));
            }
            else if self.peek(0).token == TokenType::Dec {
                self.consume();
                if self.peek(0).token != TokenType::CloseParen {
                    if self.peek(0).token != TokenType::Semi {
                        println!("excpected ;");
                    }
                    else {
                        self.consume();
                    }
                }
                self.consume();
                let dec_var = DecVar {
                    var
                };
                return  Some(Expr::DecVar(dec_var));
            }
            // function call
            else if self.peek(0).token == TokenType::OpenParen {
                let func_call = self.gen_init_func(var);
                self.consume();
                return Some(Expr::FunctionCall(func_call));
                
            }
        }
        if self.peek(0).token == TokenType::If {
            self.consume();
            let res = self.eval_expr();
            let mut expr_arr: Vec<Expr> = Vec::new(); 
            while self.peek(0).token != TokenType::CloseScope && self.peek(0).token != TokenType::Else {
                let expr = self.parse_expr().unwrap();
                expr_arr.push(expr);
            }
            if self.peek(0).token != TokenType::Else {
                    let expr = self.parse_expr().unwrap();
                    expr_arr.push(expr);
                }
            let mut else_expr_arr: Vec<Expr> = Vec::new(); 
            if self.m_tokens.len() >= 1{
                if self.peek(0).token == TokenType::Else {
                    self.consume();
                    while self.peek(0).token != TokenType::CloseScope {
                        let expr = self.parse_expr().unwrap();
                        else_expr_arr.push(expr);
                    }
                    let expr = self.parse_expr().unwrap();
                    else_expr_arr.push(expr);
                    
                }
            }
            let if_var = IfStmt {
                expr: res,
                data: expr_arr,
                else_data: else_expr_arr,
            };
            return Some(Expr::IfStmt(if_var));
        }
        if self.peek(0).token == TokenType::While {
            self.consume();
            let res = self.eval_expr();
            let mut expr_arr: Vec<Expr> = Vec::new();
            while self.peek(0).token != TokenType::CloseScope {
                let expr = self.parse_expr().unwrap();
                expr_arr.push(expr);
            }
            let while_var = WhileStmt {
                expr: res,
                data: expr_arr,
            };
            return Some(Expr::WhileStmt(while_var));
        }
        if self.peek(0).token == TokenType::For {
            self.consume();
            if self.peek(0).token != TokenType::OpenParen {
                panic!("excpected '('");
            }
            self.consume();
            let first_expr = Box::new(self.parse_expr().unwrap());
            let second_expr = self.eval_expr();
            self.consume();
            let third_expr = Box::new(self.parse_expr().unwrap());
            if self.peek(0).token != TokenType::CloseParen {
                panic!("excpected ')'");
            }
            self.consume();
            let mut expr_arr: Vec<Expr> = Vec::new(); 
            while self.peek(0).token != TokenType::CloseScope {
                let expr = self.parse_expr().unwrap();
                expr_arr.push(expr);
            }
            let for_var = ForStmt {
                expr1: first_expr,
                expr2: second_expr,
                expr3: third_expr,
                data: expr_arr,
            };
            return Some(Expr::ForStmt(for_var));
        }
        if self.peek(0).token == TokenType::Return {
            self.consume();
            let expr = self.eval_expr();
            if self.peek(0).token != TokenType::Semi {
                panic!("excpected ;");
            }
            self.consume();
            let return_ = Ret {
                expr: expr,
                func_name: self.func_name.clone(),
            };
            return Some(Expr::Ret(return_));
        }
        None
    }
}




