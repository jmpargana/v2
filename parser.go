package main

import "fmt"

/**
Grammar:

Program     := Statement*
Statement   := VarDecl | FuncDecl | ReturnStmt | ExprStmt
VarDecl     := ("let" | "const" | "var") IDENT "=" Expr ";"
FuncDecl    := "function" IDENT "(" Params? ")" "{" Statement* "}"
Params      := IDENT ("," IDENT)*
ReturnStmt  := "return" Expr ";"
ExprStmt    := Expr ";"

Expr        := Term ("+" Term)*
Term        := Factor ("*" Factor)*
Factor      := IDENT "(" Args? ")"  |  IDENT  |  NUMBER  |  "(" Expr ")"
Args        := Expr ("," Expr)*

*/

type Parser struct {
	syms []Symbol
	pos  int
}

func New(syms []Symbol) *Parser {
	return &Parser{syms: syms}
}

func (p *Parser) done() bool {
	return p.pos >= len(p.syms)
}

func (p *Parser) peek() SymbolKind {
	if p.done() {
		return EOF_TOK
	}
	return p.syms[p.pos].Kind
}

func (p *Parser) pop() Symbol {
	s := p.syms[p.pos]
	p.pos++
	return s
}

func (p *Parser) expect(kind SymbolKind) Symbol {
	if p.peek() != kind {
		panic(fmt.Sprintf("expected %s, got %s", kind, p.peek()))
	}
	return p.pop()
}

func (p *Parser) Parse() []Stmt {
	var stmts []Stmt
	for !p.done() {
		stmts = append(stmts, p.parseStmt())
	}
	return stmts
}

func (p *Parser) parseStmt() Stmt {
	switch p.peek() {
	case LET, CONST, VAR:
		return p.parseVarDecl()
	case FUNCTION:
		return p.parseFuncDecl()
	case RETURN:
		return p.parseReturnStmt()
	default:
		return p.parseExprStmt()
	}
}

func (p *Parser) parseVarDecl() *VarDecl {
	kind := p.pop().Kind
	name := p.expect(IDENT_TOK)
	p.expect(ASSIGN)
	val := p.parseExpr()
	p.expect(SEMI)
	return &VarDecl{Kind: kind, Name: name.StrVal, Value: val}
}

func (p *Parser) parseFuncDecl() *FuncDecl {
	p.expect(FUNCTION)
	name := p.expect(IDENT_TOK)
	p.expect(LPAR)
	var params []string
	if p.peek() != RPAR {
		params = append(params, p.expect(IDENT_TOK).StrVal)
		for p.peek() == COMMA {
			p.pop()
			params = append(params, p.expect(IDENT_TOK).StrVal)
		}
	}
	p.expect(RPAR)
	p.expect(LBRACE)
	var body []Stmt
	for p.peek() != RBRACE {
		body = append(body, p.parseStmt())
	}
	p.expect(RBRACE)
	return &FuncDecl{Name: name.StrVal, Params: params, Body: body}
}

func (p *Parser) parseReturnStmt() *ReturnStmt {
	p.expect(RETURN)
	val := p.parseExpr()
	p.expect(SEMI)
	return &ReturnStmt{Value: val}
}

func (p *Parser) parseExprStmt() *ExprStmt {
	expr := p.parseExpr()
	p.expect(SEMI)
	return &ExprStmt{Expr: expr}
}

func (p *Parser) parseExpr() Expr {
	left := p.parseTerm()
	for p.peek() == ADD {
		p.pop()
		right := p.parseTerm()
		left = &BinaryExpr{Op: OADD, Left: left, Right: right}
	}
	return left
}

func (p *Parser) parseTerm() Expr {
	left := p.parseFactor()
	for p.peek() == MUL {
		p.pop()
		right := p.parseFactor()
		left = &BinaryExpr{Op: OMUL, Left: left, Right: right}
	}
	return left
}

func (p *Parser) parseFactor() Expr {
	switch p.peek() {
	case SMI:
		s := p.pop()
		return &NumberLit{Value: *s.IntVal}
	case IDENT_TOK:
		s := p.pop()
		if p.peek() == LPAR {
			p.pop()
			var args []Expr
			if p.peek() != RPAR {
				args = append(args, p.parseExpr())
				for p.peek() == COMMA {
					p.pop()
					args = append(args, p.parseExpr())
				}
			}
			p.expect(RPAR)
			return &CallExpr{Func: s.StrVal, Args: args}
		}
		return &Ident{Name: s.StrVal}
	case LPAR:
		p.pop()
		expr := p.parseExpr()
		p.expect(RPAR)
		return expr
	default:
		panic(fmt.Sprintf("unexpected token in expression: %s", p.peek()))
	}
}
