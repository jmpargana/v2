use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Smi,
    Add,
    Sub,
    Mul,
    Div,
    LPar,
    RPar,
    LBrace,
    RBrace,
    Assign,
    Comma,
    Semi,
    Ident,
    Let,
    Const,
    Var,
    Function,
    Return,
    Eof,
    Lt,
    Gt,
    Equal,
    NotEqual,
    Lte,
    Gte,
    And,
    Or,
    Not,
    If,
    Else,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolKind::Smi => write!(f, "SMI"),
            SymbolKind::Add => write!(f, "+"),
            SymbolKind::Sub => write!(f, "-"),
            SymbolKind::Mul => write!(f, "*"),
            SymbolKind::Div => write!(f, "/"),
            SymbolKind::LPar => write!(f, "("),
            SymbolKind::RPar => write!(f, ")"),
            SymbolKind::LBrace => write!(f, "{{"),
            SymbolKind::RBrace => write!(f, "}}"),
            SymbolKind::Assign => write!(f, "="),
            SymbolKind::Comma => write!(f, ","),
            SymbolKind::Semi => write!(f, ";"),
            SymbolKind::Ident => write!(f, "IDENT"),
            SymbolKind::Let => write!(f, "let"),
            SymbolKind::Const => write!(f, "const"),
            SymbolKind::Var => write!(f, "var"),
            SymbolKind::Function => write!(f, "function"),
            SymbolKind::Return => write!(f, "return"),
            SymbolKind::Eof => write!(f, "EOF"),
            SymbolKind::Lt => write!(f, "<"),
            SymbolKind::Gt => write!(f, ">"),
            SymbolKind::Equal => write!(f, "=="),
            SymbolKind::NotEqual => write!(f, "!="),
            SymbolKind::Lte => write!(f, "<="),
            SymbolKind::Gte => write!(f, ">="),
            SymbolKind::And => write!(f, "&&"),
            SymbolKind::Or => write!(f, "||"),
            SymbolKind::Not => write!(f, "!"),
            SymbolKind::If => write!(f, "IF"),
            SymbolKind::Else => write!(f, "ELSE"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub kind: SymbolKind,
    pub int_val: Option<i64>,
    pub str_val: String,
}

pub struct Lexer;

impl Lexer {
    pub fn lex(src: &str) -> Vec<Symbol> {
        let chars: Vec<char> = src.chars().collect();
        let mut res = Vec::new();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch.is_whitespace() {
                i += 1;
                continue;
            }

            match ch {
                '+' => {
                    res.push(Symbol {
                        kind: SymbolKind::Add,
                        int_val: None,
                        str_val: String::new(),
                    });
                    i += 1;
                }
                '-' => {
                    res.push(Symbol {
                        kind: SymbolKind::Sub,
                        int_val: None,
                        str_val: String::new(),
                    });
                    i += 1;
                }
                '*' => {
                    res.push(Symbol {
                        kind: SymbolKind::Mul,
                        int_val: None,
                        str_val: String::new(),
                    });
                    i += 1;
                }
                '/' => {
                    res.push(Symbol {
                        kind: SymbolKind::Div,
                        int_val: None,
                        str_val: String::new(),
                    });
                    i += 1;
                }
                '<' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        res.push(Symbol {
                            kind: SymbolKind::Lte,
                            int_val: None,
                            str_val: String::new(),
                        });
                        i += 2;
                    } else {
                        res.push(Symbol {
                            kind: SymbolKind::Lt,
                            int_val: None,
                            str_val: String::new(),
                        });
                        i += 1;
                    }
                }
                '>' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        res.push(Symbol {
                            kind: SymbolKind::Gte,
                            int_val: None,
                            str_val: String::new(),
                        });
                        i += 2;
                    } else {
                        res.push(Symbol {
                            kind: SymbolKind::Gt,
                            int_val: None,
                            str_val: String::new(),
                        });
                        i += 1;
                    }
                }
                '!' => {
                    if i + 1 < chars.len() && chars[i + 1] == '=' {
                        res.push(Symbol {
                            kind: SymbolKind::NotEqual,
                            int_val: None,
                            str_val: String::new(),
                        });
                        i += 2;
                    } else {
                        res.push(Symbol {
                            kind: SymbolKind::Not,
                            int_val: None,
                            str_val: String::new(),
                        });
                        i += 1;
                    }
                }
                '&' => {
                    if i + 1 < chars.len() && chars[i + 1] == '&' {
                        res.push(Symbol {
                            kind: SymbolKind::And,
                            int_val: None,
                            str_val: String::new(),
                        });
                        i += 2;
                    } else {
                        panic!("unexpected character: &");
                    }
                }
                '|' => {
                    if i + 1 < chars.len() && chars[i + 1] == '|' {
                        res.push(Symbol {
                            kind: SymbolKind::Or,
                            int_val: None,
                            str_val: String::new(),
                        });
                        i += 2;
                    } else {
                        panic!("unexpected character: |");
                    }
                }
                '(' => {
                    res.push(Symbol {
                        kind: SymbolKind::LPar,
                        int_val: None,
                        str_val: String::new(),
                    });
                    i += 1;
                }
                ')' => {
                    res.push(Symbol {
                        kind: SymbolKind::RPar,
                        int_val: None,
                        str_val: String::new(),
                    });
                    i += 1;
                }
                '{' => {
                    res.push(Symbol {
                        kind: SymbolKind::LBrace,
                        int_val: None,
                        str_val: String::new(),
                    });
                    i += 1;
                }
                '}' => {
                    res.push(Symbol {
                        kind: SymbolKind::RBrace,
                        int_val: None,
                        str_val: String::new(),
                    });
                    i += 1;
                }
                '=' => {
                    if chars[i + 1] == '=' {
                        res.push(Symbol {
                            kind: SymbolKind::Equal,
                            int_val: None,
                            str_val: String::new(),
                        });
                        i += 2;
                    } else {
                        res.push(Symbol {
                            kind: SymbolKind::Assign,
                            int_val: None,
                            str_val: String::new(),
                        });
                        i += 1;
                    }
                }
                ',' => {
                    res.push(Symbol {
                        kind: SymbolKind::Comma,
                        int_val: None,
                        str_val: String::new(),
                    });
                    i += 1;
                }
                ';' => {
                    res.push(Symbol {
                        kind: SymbolKind::Semi,
                        int_val: None,
                        str_val: String::new(),
                    });
                    i += 1;
                }
                _ if ch.is_ascii_digit() => {
                    let start = i;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    let n: i64 = chars[start..i].iter().collect::<String>().parse().unwrap();
                    res.push(Symbol {
                        kind: SymbolKind::Smi,
                        int_val: Some(n),
                        str_val: String::new(),
                    });
                }
                _ if ch.is_alphabetic() || ch == '_' => {
                    let start = i;
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    let word: String = chars[start..i].iter().collect();
                    match word.as_str() {
                        "let" => res.push(Symbol {
                            kind: SymbolKind::Let,
                            int_val: None,
                            str_val: String::new(),
                        }),
                        "const" => res.push(Symbol {
                            kind: SymbolKind::Const,
                            int_val: None,
                            str_val: String::new(),
                        }),
                        "var" => res.push(Symbol {
                            kind: SymbolKind::Var,
                            int_val: None,
                            str_val: String::new(),
                        }),
                        "if" => res.push(Symbol {
                            kind: SymbolKind::If,
                            int_val: None,
                            str_val: String::new(),
                        }),
                        "else" => res.push(Symbol {
                            kind: SymbolKind::Else,
                            int_val: None,
                            str_val: String::new(),
                        }),
                        "function" => res.push(Symbol {
                            kind: SymbolKind::Function,
                            int_val: None,
                            str_val: String::new(),
                        }),
                        "return" => res.push(Symbol {
                            kind: SymbolKind::Return,
                            int_val: None,
                            str_val: String::new(),
                        }),
                        _ => res.push(Symbol {
                            kind: SymbolKind::Ident,
                            int_val: None,
                            str_val: word,
                        }),
                    }
                }
                _ => panic!("unexpected character: {}", ch),
            }
        }

        res
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
    fn single_number() {
        assert_eq!(Lexer::lex("123"), vec![smi(123)]);
    }

    #[test]
    fn arithmetic_expression() {
        assert_eq!(
            Lexer::lex("123 + (23 * 43) * 3"),
            vec![
                smi(123),
                tok(SymbolKind::Add),
                tok(SymbolKind::LPar),
                smi(23),
                tok(SymbolKind::Mul),
                smi(43),
                tok(SymbolKind::RPar),
                tok(SymbolKind::Mul),
                smi(3),
            ]
        );
    }

    #[test]
    fn variable_declaration() {
        assert_eq!(
            Lexer::lex("let x = 5;"),
            vec![
                tok(SymbolKind::Let),
                ident("x"),
                tok(SymbolKind::Assign),
                smi(5),
                tok(SymbolKind::Semi),
            ]
        );
    }

    #[test]
    fn condition() {
        assert_eq!(
            Lexer::lex("a == b"),
            vec![ident("a"), tok(SymbolKind::Equal), ident("b"),]
        );
    }

    #[test]
    fn function_declaration() {
        assert_eq!(
            Lexer::lex("function add(a, b) { return a + b; }"),
            vec![
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
            ]
        );
    }

    #[test]
    fn const_and_var_keywords() {
        assert_eq!(
            Lexer::lex("const x = 1; var y = 2;"),
            vec![
                tok(SymbolKind::Const),
                ident("x"),
                tok(SymbolKind::Assign),
                smi(1),
                tok(SymbolKind::Semi),
                tok(SymbolKind::Var),
                ident("y"),
                tok(SymbolKind::Assign),
                smi(2),
                tok(SymbolKind::Semi),
            ]
        );
    }

    #[test]
    fn identifier_vs_keyword() {
        assert_eq!(
            Lexer::lex("let letters = 42;"),
            vec![
                tok(SymbolKind::Let),
                ident("letters"),
                tok(SymbolKind::Assign),
                smi(42),
                tok(SymbolKind::Semi),
            ]
        );
    }
}
