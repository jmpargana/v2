package main

type Parser struct {
	syms []Symbol
}

/**
	Expr := Res Op Res
	Res := (Expr) | Digit
	Op := + | *

*/

func New(syms []Symbol) *Parser {
	return &Parser{
		syms: syms,
	}
}

func (p *Parser) Peek() *Symbol {
	return &p.syms[0]
}

func (p *Parser) Pop() Symbol {
	el := p.syms[0]
	p.syms = p.syms[1:]
	return el
}

func (p *Parser) Parse() *AST {
	return p.parseExpr()
}

func (p *Parser) parseOp() Op {
	switch p.Pop().Kind {
	case MUL:
		return OMUL
	default:
		return OADD
	}
}

func (p *Parser) parseExpr() *AST {
	left := p.parseRes()

    if len(p.syms) == 0 {
        return left
    }	

	op := p.parseOp()
	right := p.parseRes()

	return &AST{
		Left:  left,
		Op:    op,
		Right: right,
	}
}

func (p *Parser) parseRes() *AST {
	if p.Peek().Kind == LPAR {
		p.Pop()

		ast := p.parseExpr()

		if p.Peek().Kind != RPAR {
			panic("expected ')'")
		}

		p.Pop()

		return ast
	}

	v := p.Pop()
	return &AST{
		Value: v.Value,
	}
}