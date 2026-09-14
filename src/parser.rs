use crate::lex::{Symbol, SymbolKind};
use crate::ast::{Expr, Stmt, Op};

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
            SymbolKind::Function => self.parse_func_decl(),
            SymbolKind::Return => self.parse_return_stmt(),
            _ => self.parse_expr_stmt(),
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
        let expr = self.parse_expr();
        self.expect(SymbolKind::Semi);
        Stmt::ExprStmt(expr)
    }

    fn parse_expr(&mut self) -> Expr {
        let mut left = self.parse_term();
        while self.peek() == SymbolKind::Add {
            self.pop();
            let right = self.parse_term();
            left = Expr::Binary {
                op: Op::Add,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        while self.peek() == SymbolKind::Mul {
            self.pop();
            let right = self.parse_factor();
            left = Expr::Binary {
                op: Op::Mul,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_factor(&mut self) -> Expr {
        match self.peek() {
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
                    Expr::Call { func: s.str_val, args }
                } else {
                    Expr::Ident(s.str_val)
                }
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
        Symbol { kind: SymbolKind::Smi, int_val: Some(val), str_val: String::new() }
    }

    fn tok(kind: SymbolKind) -> Symbol {
        Symbol { kind, int_val: None, str_val: String::new() }
    }

    fn ident(name: &str) -> Symbol {
        Symbol { kind: SymbolKind::Ident, int_val: None, str_val: name.to_string() }
    }

    #[test]
    fn expression_statement() {
        let got = Parser::new(vec![smi(30), tok(SymbolKind::Semi)]).parse();
        assert_eq!(got, vec![Stmt::ExprStmt(Expr::NumberLit(30))]);
    }

    #[test]
    fn binary_expression_with_precedence() {
        let got = Parser::new(vec![
            smi(1), tok(SymbolKind::Add), smi(2), tok(SymbolKind::Mul), smi(3), tok(SymbolKind::Semi),
        ]).parse();
        assert_eq!(got, vec![
            Stmt::ExprStmt(Expr::Binary {
                op: Op::Add,
                left: Box::new(Expr::NumberLit(1)),
                right: Box::new(Expr::Binary {
                    op: Op::Mul,
                    left: Box::new(Expr::NumberLit(2)),
                    right: Box::new(Expr::NumberLit(3)),
                }),
            }),
        ]);
    }

    #[test]
    fn parenthesized_expression() {
        let got = Parser::new(vec![
            smi(30), tok(SymbolKind::Add), tok(SymbolKind::LPar),
            smi(20), tok(SymbolKind::Mul), smi(40), tok(SymbolKind::RPar),
            tok(SymbolKind::Semi),
        ]).parse();
        assert_eq!(got, vec![
            Stmt::ExprStmt(Expr::Binary {
                op: Op::Add,
                left: Box::new(Expr::NumberLit(30)),
                right: Box::new(Expr::Binary {
                    op: Op::Mul,
                    left: Box::new(Expr::NumberLit(20)),
                    right: Box::new(Expr::NumberLit(40)),
                }),
            }),
        ]);
    }

    #[test]
    fn variable_declaration() {
        let got = Parser::new(vec![
            tok(SymbolKind::Let), ident("x"), tok(SymbolKind::Assign),
            smi(5), tok(SymbolKind::Add), smi(3), tok(SymbolKind::Semi),
        ]).parse();
        assert_eq!(got, vec![
            Stmt::VarDecl {
                kind: SymbolKind::Let,
                name: "x".to_string(),
                value: Expr::Binary {
                    op: Op::Add,
                    left: Box::new(Expr::NumberLit(5)),
                    right: Box::new(Expr::NumberLit(3)),
                },
            },
        ]);
    }

    #[test]
    fn function_declaration_with_return() {
        let got = Parser::new(vec![
            tok(SymbolKind::Function), ident("add"), tok(SymbolKind::LPar),
            ident("a"), tok(SymbolKind::Comma), ident("b"), tok(SymbolKind::RPar),
            tok(SymbolKind::LBrace),
            tok(SymbolKind::Return), ident("a"), tok(SymbolKind::Add), ident("b"), tok(SymbolKind::Semi),
            tok(SymbolKind::RBrace),
        ]).parse();
        assert_eq!(got, vec![
            Stmt::FuncDecl {
                name: "add".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                body: vec![
                    Stmt::Return(Expr::Binary {
                        op: Op::Add,
                        left: Box::new(Expr::Ident("a".to_string())),
                        right: Box::new(Expr::Ident("b".to_string())),
                    }),
                ],
            },
        ]);
    }

    #[test]
    fn function_call() {
        let got = Parser::new(vec![
            ident("add"), tok(SymbolKind::LPar),
            smi(1), tok(SymbolKind::Comma), smi(2),
            tok(SymbolKind::RPar), tok(SymbolKind::Semi),
        ]).parse();
        assert_eq!(got, vec![
            Stmt::ExprStmt(Expr::Call {
                func: "add".to_string(),
                args: vec![Expr::NumberLit(1), Expr::NumberLit(2)],
            }),
        ]);
    }

    #[test]
    fn no_arg_function() {
        let got = Parser::new(vec![
            tok(SymbolKind::Function), ident("noop"),
            tok(SymbolKind::LPar), tok(SymbolKind::RPar),
            tok(SymbolKind::LBrace), tok(SymbolKind::RBrace),
        ]).parse();
        assert_eq!(got, vec![
            Stmt::FuncDecl {
                name: "noop".to_string(),
                params: vec![],
                body: vec![],
            },
        ]);
    }

    #[test]
    fn multi_statement_program() {
        let got = Parser::new(vec![
            tok(SymbolKind::Let), ident("x"), tok(SymbolKind::Assign),
            smi(10), tok(SymbolKind::Semi),
            tok(SymbolKind::Let), ident("y"), tok(SymbolKind::Assign),
            ident("x"), tok(SymbolKind::Mul), smi(2), tok(SymbolKind::Semi),
        ]).parse();
        assert_eq!(got, vec![
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
        ]);
    }
}
