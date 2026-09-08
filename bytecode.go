package main

import (
	"fmt"
	"strings"
)

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
	OpLdaSmi
	OpLdar
	OpStar
)

func (op ByteCode) String() string {
	switch op {
	case OpPush:
		return "Push"
	case OpAdd:
		return "Add"
	case OpMul:
		return "Mul"
	case OpLdaSmi:
		return "LdaSmi"
	case OpLdar:
		return "Ldar"
	case OpStar:
		return "Star"
	default:
		return fmt.Sprintf("Unknown(%d)", op)
	}
}

func (p *Program) String() string {
	var b strings.Builder
	fmt.Fprintf(&b, "Constants: %v\n", p.cons)
	fmt.Fprintln(&b, "Bytecode:")
	for i := 0; i < len(p.code); {
		op := ByteCode(p.code[i])
		if i+1 < len(p.code) {
			operand := p.code[i+1]
			switch op {
			case OpLdaSmi, OpPush:
				if int(operand) < len(p.cons) {
					fmt.Fprintf(&b, "  %04d  %-8s [%d] (%d)\n", i, op, operand, p.cons[operand])
				} else {
					fmt.Fprintf(&b, "  %04d  %-8s [%d]\n", i, op, operand)
				}
			case OpStar, OpLdar, OpAdd, OpMul:
				fmt.Fprintf(&b, "  %04d  %-8s r%d\n", i, op, operand)
			default:
				fmt.Fprintf(&b, "  %04d  %-8s %d\n", i, op, operand)
			}
			i += 2
		} else {
			fmt.Fprintf(&b, "  %04d  %s\n", i, op)
			i++
		}
	}
	return b.String()
}

type Program struct {
	code []byte
	cons []int
	currReg int
}

func (p *Program) alloReg() int {
	val := p.currReg	
	// ring buffer
	p.currReg = (p.currReg + 1) % 256
	return val
}


func (p *Program) Compile(ast *AST) *Program {
	if ast.Value != nil {
		idx := len(p.cons)
		p.cons = append(p.cons, *ast.Value)
		p.code = append(p.code, byte(OpLdaSmi), byte(idx))
		return p
	}
	
	p.Compile(ast.Left)
	
	reg := p.alloReg()
	p.code = append(p.code, byte(OpStar), byte(reg))
	
	p.Compile(ast.Right)
	
	switch ast.Op {
	case OMUL:
		p.code = append(p.code, byte(OpMul), byte(reg))
	case OADD:
		p.code = append(p.code, byte(OpAdd), byte(reg))
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