package main

/**

Step-by-step progression:

1. Stack Machine
2. Register

*/

type ByteCode byte

const (
	OpPush ByteCode = iota
	OpAdd
	OpMul
)

type Program struct {
	code []ByteCode
	cons []int
}

func (p *Program) Compile(ast *AST) *Program {
	if ast.Value != nil {
		p.cons = append(p.cons, *ast.Value)
		p.code = append(p.code, OpPush)
		return p
	}
	
	p.Compile(ast.Left)
	p.Compile(ast.Right)
	
	switch ast.Op {
	case OMUL:
		p.code = append(p.code, OpMul)
	case OADD:
		p.code = append(p.code, OpAdd)
	}
	
	return p
}

func (p *Program) Equals(other *Program) bool {
	if len(p.code) != len(other.code) || len(p.cons) != len(other.cons) {
		return false
	}

	for i, op := range p.code {
		if op != other.code[i] {
			return false
		}
	}

	for i, c := range p.cons {
		if c != other.cons[i] {
			return false
		}
	}

	return true
}