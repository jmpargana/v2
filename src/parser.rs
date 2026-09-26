use crate::ast::{Expr, Op, Stmt};
use crate::lex::{Symbol, SymbolKind};

pub struct Parser {
    syms: Vec<Symbol>,
    pos: usize,
}

impl Parser {
    pub fn new(syms: Vec<Symbol>) -> Self {
        Parser { syms, pos: 0 }
    }

    fn done(&self) -> bool {
        self.pos >= self.syms.len()
    }

    fn peek(&self) -> SymbolKind {
        if self.done() {
            SymbolKind::Eof
        } else {
            self.syms[self.pos].kind
        }
    }

    fn pop(&mut self) -> Symbol {
        let s = self.syms[self.pos].clone();
        self.pos += 1;
        s
    }

    fn expect(&mut self, kind: SymbolKind) -> Symbol {
        if self.peek() != kind {
            panic!("expected {}, got {}", kind, self.peek());
        }
        self.pop()
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.done() {
            stmts.push(self.parse_stmt());
        }
        stmts
    }

    fn parse_stmt(&mut self) -> Stmt {
        match self.peek() {
            SymbolKind::Let | SymbolKind::Const | SymbolKind::Var => self.parse_var_decl(),
            SymbolKind::If => self.parse_conditional(),
            SymbolKind::While => self.parse_while_loop(),
            SymbolKind::For => self.parse_for_loop(),
            SymbolKind::Function => self.parse_func_decl(),
            SymbolKind::Return => self.parse_return_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_while_loop(&mut self) -> Stmt {
        self.expect(SymbolKind::While);

        self.expect(SymbolKind::LPar);
        let condition = self.parse_expr();
        self.expect(SymbolKind::RPar);

        self.expect(SymbolKind::LBrace);
        let mut body = Vec::new();
        while self.peek() != SymbolKind::RBrace {
            body.push(self.parse_stmt());
        }
        self.expect(SymbolKind::RBrace);

        Stmt::WhileLoop { condition, body }
    }

    fn parse_for_loop(&mut self) -> Stmt {
        self.expect(SymbolKind::For);

        self.expect(SymbolKind::LPar);
        let init = self.parse_stmt();
        let condition = self.parse_expr();
        self.expect(SymbolKind::Semi);
        let update = self.parse_for_update();
        self.expect(SymbolKind::RPar);

        self.expect(SymbolKind::LBrace);
        let mut body = Vec::new();
        while self.peek() != SymbolKind::RBrace {
            body.push(self.parse_stmt());
        }
        self.expect(SymbolKind::RBrace);

        Stmt::ForLoop {
            init: Box::new(init),
            condition,
            body,
            update: Box::new(update),
        }
    }

    fn parse_conditional(&mut self) -> Stmt {
        self.expect(SymbolKind::If);

        self.expect(SymbolKind::LPar);
        let condition = self.parse_expr();
        self.expect(SymbolKind::RPar);

        self.expect(SymbolKind::LBrace);
        let mut body = Vec::new();
        while self.peek() != SymbolKind::RBrace {
            body.push(self.parse_stmt());
        }
        self.expect(SymbolKind::RBrace);

        let mut alternate = None;
        if self.peek() == SymbolKind::Else {
            self.pop();
            self.expect(SymbolKind::LBrace);
            let mut alternate_body = Vec::new();
            while self.peek() != SymbolKind::RBrace {
                alternate_body.push(self.parse_stmt());
            }
            self.expect(SymbolKind::RBrace);
            alternate = Some(alternate_body);
        }

        Stmt::Cond {
            condition,
            body,
            alternate,
        }
    }

    fn parse_var_decl(&mut self) -> Stmt {
        let kind = self.pop().kind;
        let name = self.expect(SymbolKind::Ident).str_val;
        self.expect(SymbolKind::Assign);
        let value = self.parse_expr();
        self.expect(SymbolKind::Semi);
        Stmt::VarDecl { kind, name, value }
    }

    fn parse_func_decl(&mut self) -> Stmt {
        self.expect(SymbolKind::Function);
        let name = self.expect(SymbolKind::Ident).str_val;
        self.expect(SymbolKind::LPar);
        let mut params = Vec::new();
        if self.peek() != SymbolKind::RPar {
            params.push(self.expect(SymbolKind::Ident).str_val);
            while self.peek() == SymbolKind::Comma {
                self.pop();
                params.push(self.expect(SymbolKind::Ident).str_val);
            }
        }
        self.expect(SymbolKind::RPar);
        self.expect(SymbolKind::LBrace);
        let mut body = Vec::new();
        while self.peek() != SymbolKind::RBrace {
            body.push(self.parse_stmt());
        }
        self.expect(SymbolKind::RBrace);
        Stmt::FuncDecl { name, params, body }
    }

    fn parse_return_stmt(&mut self) -> Stmt {
        self.expect(SymbolKind::Return);
        let value = self.parse_expr();
        self.expect(SymbolKind::Semi);
        Stmt::Return(value)
    }

    fn parse_expr_stmt(&mut self) -> Stmt {
        if self.peek() == SymbolKind::Ident
            && self.pos + 1 < self.syms.len()
            && self.syms[self.pos + 1].kind == SymbolKind::Assign
        {
            let name = self.pop().str_val;
            self.expect(SymbolKind::Assign);
            let value = self.parse_expr();
            self.expect(SymbolKind::Semi);
            return Stmt::Assign { name, value };
        }
        let expr = self.parse_expr();
        self.expect(SymbolKind::Semi);
        Stmt::ExprStmt(expr)
    }

    fn parse_for_update(&mut self) -> Stmt {
        if self.peek() == SymbolKind::Ident
            && self.pos + 1 < self.syms.len()
            && self.syms[self.pos + 1].kind == SymbolKind::Assign
        {
            let name = self.pop().str_val;
            self.expect(SymbolKind::Assign);
            let value = self.parse_expr();
            return Stmt::Assign { name, value };
        }
        let expr = self.parse_expr();
        Stmt::ExprStmt(expr)
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Expr {
        let mut left = self.parse_logical_and();
        while self.peek() == SymbolKind::Or {
            self.pop();
            let right = self.parse_logical_and();
            left = Expr::Bool {
                op: Op::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_logical_and(&mut self) -> Expr {
        let mut left = self.parse_comparison();
        while self.peek() == SymbolKind::And {
            self.pop();
            let right = self.parse_comparison();
            left = Expr::Bool {
                op: Op::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_comparison(&mut self) -> Expr {
        let left = self.parse_additive();
        match self.peek() {
            SymbolKind::Lt
            | SymbolKind::Gt
            | SymbolKind::Equal
            | SymbolKind::Lte
            | SymbolKind::Gte
            | SymbolKind::NotEqual => {
                let op = match self.pop().kind {
                    SymbolKind::Lt => Op::Lt,
                    SymbolKind::Gt => Op::Gt,
                    SymbolKind::Equal => Op::Equal,
                    SymbolKind::Lte => Op::Lte,
                    SymbolKind::Gte => Op::Gte,
                    SymbolKind::NotEqual => Op::NotEqual,
                    _ => unreachable!(),
                };
                let right = self.parse_additive();
                Expr::Bool {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }
            _ => left,
        }
    }

    fn parse_additive(&mut self) -> Expr {
        let mut left = self.parse_term();
        while matches!(self.peek(), SymbolKind::Add | SymbolKind::Sub) {
            let op = if self.pop().kind == SymbolKind::Add {
                Op::Add
            } else {
                Op::Sub
            };
            let right = self.parse_term();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        while matches!(self.peek(), SymbolKind::Mul | SymbolKind::Div) {
            let op = if self.pop().kind == SymbolKind::Mul {
                Op::Mul
            } else {
                Op::Div
            };
            let right = self.parse_factor();
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_factor(&mut self) -> Expr {
        match self.peek() {
            SymbolKind::Str => {
                let s = self.pop();
                Expr::StringLit(s.str_val)
            }
            SymbolKind::Smi => {
                let s = self.pop();
                Expr::NumberLit(s.int_val.unwrap())
            }
            SymbolKind::Ident => {
                let s = self.pop();
                if self.peek() == SymbolKind::LPar {
                    self.pop();
                    let mut args = Vec::new();
                    if self.peek() != SymbolKind::RPar {
                        args.push(self.parse_expr());
                        while self.peek() == SymbolKind::Comma {
                            self.pop();
                            args.push(self.parse_expr());
                        }
                    }
                    self.expect(SymbolKind::RPar);
                    Expr::Call {
                        func: s.str_val,
                        args,
                    }
                } else {
                    Expr::Ident(s.str_val)
                }
            }
            SymbolKind::Not => {
                self.pop();
                let operand = self.parse_factor();
                Expr::Not(Box::new(operand))
            }
            SymbolKind::LPar => {
                self.pop();
                let expr = self.parse_expr();
                self.expect(SymbolKind::RPar);
                expr
            }
            _ => panic!("unexpected token in expression: {}", self.peek()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn smi(val: i64) -> Symbol {
        Symbol {
            kind: SymbolKind::Smi,
            int_val: Some(val),
            str_val: String::new(),
        }
    }

    fn tok(kind: SymbolKind) -> Symbol {
        Symbol {
            kind,
            int_val: None,
            str_val: String::new(),
        }
    }

    fn ident(name: &str) -> Symbol {
        Symbol {
            kind: SymbolKind::Ident,
            int_val: None,
            str_val: name.to_string(),
        }
    }

    #[test]
    fn expression_statement() {
        let got = Parser::new(vec![smi(30), tok(SymbolKind::Semi)]).parse();
        assert_eq!(got, vec![Stmt::ExprStmt(Expr::NumberLit(30))]);
    }

    #[test]
    fn binary_expression_with_precedence() {
        let got = Parser::new(vec![
            smi(1),
            tok(SymbolKind::Add),
            smi(2),
            tok(SymbolKind::Mul),
            smi(3),
            tok(SymbolKind::Semi),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::ExprStmt(Expr::Binary {
                op: Op::Add,
                left: Box::new(Expr::NumberLit(1)),
                right: Box::new(Expr::Binary {
                    op: Op::Mul,
                    left: Box::new(Expr::NumberLit(2)),
                    right: Box::new(Expr::NumberLit(3)),
                }),
            }),]
        );
    }

    #[test]
    fn parenthesized_expression() {
        let got = Parser::new(vec![
            smi(30),
            tok(SymbolKind::Add),
            tok(SymbolKind::LPar),
            smi(20),
            tok(SymbolKind::Mul),
            smi(40),
            tok(SymbolKind::RPar),
            tok(SymbolKind::Semi),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::ExprStmt(Expr::Binary {
                op: Op::Add,
                left: Box::new(Expr::NumberLit(30)),
                right: Box::new(Expr::Binary {
                    op: Op::Mul,
                    left: Box::new(Expr::NumberLit(20)),
                    right: Box::new(Expr::NumberLit(40)),
                }),
            }),]
        );
    }

    #[test]
    fn variable_declaration() {
        let got = Parser::new(vec![
            tok(SymbolKind::Let),
            ident("x"),
            tok(SymbolKind::Assign),
            smi(5),
            tok(SymbolKind::Add),
            smi(3),
            tok(SymbolKind::Semi),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::VarDecl {
                kind: SymbolKind::Let,
                name: "x".to_string(),
                value: Expr::Binary {
                    op: Op::Add,
                    left: Box::new(Expr::NumberLit(5)),
                    right: Box::new(Expr::NumberLit(3)),
                },
            },]
        );
    }

    #[test]
    fn function_declaration_with_return() {
        let got = Parser::new(vec![
            tok(SymbolKind::Function),
            ident("add"),
            tok(SymbolKind::LPar),
            ident("a"),
            tok(SymbolKind::Comma),
            ident("b"),
            tok(SymbolKind::RPar),
            tok(SymbolKind::LBrace),
            tok(SymbolKind::Return),
            ident("a"),
            tok(SymbolKind::Add),
            ident("b"),
            tok(SymbolKind::Semi),
            tok(SymbolKind::RBrace),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::FuncDecl {
                name: "add".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                body: vec![Stmt::Return(Expr::Binary {
                    op: Op::Add,
                    left: Box::new(Expr::Ident("a".to_string())),
                    right: Box::new(Expr::Ident("b".to_string())),
                }),],
            },]
        );
    }

    #[test]
    fn function_call() {
        let got = Parser::new(vec![
            ident("add"),
            tok(SymbolKind::LPar),
            smi(1),
            tok(SymbolKind::Comma),
            smi(2),
            tok(SymbolKind::RPar),
            tok(SymbolKind::Semi),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::ExprStmt(Expr::Call {
                func: "add".to_string(),
                args: vec![Expr::NumberLit(1), Expr::NumberLit(2)],
            }),]
        );
    }

    #[test]
    fn no_arg_function() {
        let got = Parser::new(vec![
            tok(SymbolKind::Function),
            ident("noop"),
            tok(SymbolKind::LPar),
            tok(SymbolKind::RPar),
            tok(SymbolKind::LBrace),
            tok(SymbolKind::RBrace),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::FuncDecl {
                name: "noop".to_string(),
                params: vec![],
                body: vec![],
            },]
        );
    }

    #[test]
    fn boolean_expression_greater_than() {
        let got = Parser::new(vec![
            ident("a"),
            tok(SymbolKind::Gt),
            ident("b"),
            tok(SymbolKind::Semi),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::ExprStmt(Expr::Bool {
                op: Op::Gt,
                left: Box::new(Expr::Ident("a".to_string())),
                right: Box::new(Expr::Ident("b".to_string())),
            })]
        );
    }

    #[test]
    fn boolean_expression_less_than() {
        let got = Parser::new(vec![
            smi(1),
            tok(SymbolKind::Lt),
            smi(2),
            tok(SymbolKind::Semi),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::ExprStmt(Expr::Bool {
                op: Op::Lt,
                left: Box::new(Expr::NumberLit(1)),
                right: Box::new(Expr::NumberLit(2)),
            })]
        );
    }

    #[test]
    fn boolean_expression_equal() {
        let got = Parser::new(vec![
            ident("x"),
            tok(SymbolKind::Equal),
            smi(10),
            tok(SymbolKind::Semi),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::ExprStmt(Expr::Bool {
                op: Op::Equal,
                left: Box::new(Expr::Ident("x".to_string())),
                right: Box::new(Expr::NumberLit(10)),
            })]
        );
    }

    #[test]
    fn comparison_with_arithmetic() {
        // a + 1 > b * 2;
        let got = Parser::new(vec![
            ident("a"),
            tok(SymbolKind::Add),
            smi(1),
            tok(SymbolKind::Gt),
            ident("b"),
            tok(SymbolKind::Mul),
            smi(2),
            tok(SymbolKind::Semi),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::ExprStmt(Expr::Bool {
                op: Op::Gt,
                left: Box::new(Expr::Binary {
                    op: Op::Add,
                    left: Box::new(Expr::Ident("a".to_string())),
                    right: Box::new(Expr::NumberLit(1)),
                }),
                right: Box::new(Expr::Binary {
                    op: Op::Mul,
                    left: Box::new(Expr::Ident("b".to_string())),
                    right: Box::new(Expr::NumberLit(2)),
                }),
            })]
        );
    }

    #[test]
    fn conditional_without_else() {
        // if (a > b) { return a; }
        let got = Parser::new(vec![
            tok(SymbolKind::If),
            tok(SymbolKind::LPar),
            ident("a"),
            tok(SymbolKind::Gt),
            ident("b"),
            tok(SymbolKind::RPar),
            tok(SymbolKind::LBrace),
            tok(SymbolKind::Return),
            ident("a"),
            tok(SymbolKind::Semi),
            tok(SymbolKind::RBrace),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::Cond {
                condition: Expr::Bool {
                    op: Op::Gt,
                    left: Box::new(Expr::Ident("a".to_string())),
                    right: Box::new(Expr::Ident("b".to_string())),
                },
                body: vec![Stmt::Return(Expr::Ident("a".to_string()))],
                alternate: None,
            }]
        );
    }

    #[test]
    fn conditional_with_else() {
        // if (a > b) { return a; } else { return b; }
        let got = Parser::new(vec![
            tok(SymbolKind::If),
            tok(SymbolKind::LPar),
            ident("a"),
            tok(SymbolKind::Gt),
            ident("b"),
            tok(SymbolKind::RPar),
            tok(SymbolKind::LBrace),
            tok(SymbolKind::Return),
            ident("a"),
            tok(SymbolKind::Semi),
            tok(SymbolKind::RBrace),
            tok(SymbolKind::Else),
            tok(SymbolKind::LBrace),
            tok(SymbolKind::Return),
            ident("b"),
            tok(SymbolKind::Semi),
            tok(SymbolKind::RBrace),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::Cond {
                condition: Expr::Bool {
                    op: Op::Gt,
                    left: Box::new(Expr::Ident("a".to_string())),
                    right: Box::new(Expr::Ident("b".to_string())),
                },
                body: vec![Stmt::Return(Expr::Ident("a".to_string()))],
                alternate: Some(vec![Stmt::Return(Expr::Ident("b".to_string()))]),
            }]
        );
    }

    fn str_lit(val: &str) -> Symbol {
        Symbol {
            kind: SymbolKind::Str,
            int_val: None,
            str_val: val.to_string(),
        }
    }

    #[test]
    fn string_literal_expression() {
        let got = Parser::new(vec![str_lit("hello"), tok(SymbolKind::Semi)]).parse();
        assert_eq!(
            got,
            vec![Stmt::ExprStmt(Expr::StringLit("hello".to_string()))]
        );
    }

    #[test]
    fn string_var_decl() {
        let got = Parser::new(vec![
            tok(SymbolKind::Let),
            ident("name"),
            tok(SymbolKind::Assign),
            str_lit("alice"),
            tok(SymbolKind::Semi),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::VarDecl {
                kind: SymbolKind::Let,
                name: "name".to_string(),
                value: Expr::StringLit("alice".to_string()),
            }]
        );
    }

    #[test]
    fn string_equality_comparison() {
        let got = Parser::new(vec![
            str_lit("a"),
            tok(SymbolKind::Equal),
            str_lit("b"),
            tok(SymbolKind::Semi),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::ExprStmt(Expr::Bool {
                op: Op::Equal,
                left: Box::new(Expr::StringLit("a".to_string())),
                right: Box::new(Expr::StringLit("b".to_string())),
            })]
        );
    }

    #[test]
    fn string_as_function_argument() {
        let got = Parser::new(vec![
            ident("greet"),
            tok(SymbolKind::LPar),
            str_lit("world"),
            tok(SymbolKind::RPar),
            tok(SymbolKind::Semi),
        ])
        .parse();
        assert_eq!(
            got,
            vec![Stmt::ExprStmt(Expr::Call {
                func: "greet".to_string(),
                args: vec![Expr::StringLit("world".to_string())],
            })]
        );
    }

    #[test]
    fn multi_statement_program() {
        let got = Parser::new(vec![
            tok(SymbolKind::Let),
            ident("x"),
            tok(SymbolKind::Assign),
            smi(10),
            tok(SymbolKind::Semi),
            tok(SymbolKind::Let),
            ident("y"),
            tok(SymbolKind::Assign),
            ident("x"),
            tok(SymbolKind::Mul),
            smi(2),
            tok(SymbolKind::Semi),
        ])
        .parse();
        assert_eq!(
            got,
            vec![
                Stmt::VarDecl {
                    kind: SymbolKind::Let,
                    name: "x".to_string(),
                    value: Expr::NumberLit(10),
                },
                Stmt::VarDecl {
                    kind: SymbolKind::Let,
                    name: "y".to_string(),
                    value: Expr::Binary {
                        op: Op::Mul,
                        left: Box::new(Expr::Ident("x".to_string())),
                        right: Box::new(Expr::NumberLit(2)),
                    },
                },
            ]
        );
    }
}
