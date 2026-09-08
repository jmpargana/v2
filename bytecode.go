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
	OpCall
	OpReturn
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
	case OpCall:
		return "Call"
	case OpReturn:
		return "Return"
	default:
		return fmt.Sprintf("Unknown(%d)", op)
	}
}

func (p *Program) stringIndent(indent string) string {
	var b strings.Builder
	fmt.Fprintf(&b, "%sConstants: %v\n", indent, p.cons)
	fmt.Fprintf(&b, "%sRegisters: %d, Params: %d\n", indent, p.nextReg, p.paramCount)
	if len(p.funcMap) > 0 {
		fmt.Fprintf(&b, "%sFuncMap: %v\n", indent, p.funcMap)
	}
	fmt.Fprintf(&b, "%sBytecode:\n", indent)
	for i := 0; i < len(p.code); {
		op := ByteCode(p.code[i])
		switch op {
		case OpReturn:
			fmt.Fprintf(&b, "%s  %04d  %s\n", indent, i, op)
			i++
		default:
			if i+1 < len(p.code) {
				operand := p.code[i+1]
				switch op {
				case OpLdaSmi, OpPush:
					if int(operand) < len(p.cons) {
						fmt.Fprintf(&b, "%s  %04d  %-8s [%d] (%d)\n", indent, i, op, operand, p.cons[operand])
					} else {
						fmt.Fprintf(&b, "%s  %04d  %-8s [%d]\n", indent, i, op, operand)
					}
				case OpStar, OpLdar, OpAdd, OpMul:
					fmt.Fprintf(&b, "%s  %04d  %-8s r%d\n", indent, i, op, operand)
				case OpCall:
					name := ""
					for k, v := range p.funcMap {
						if v == int(operand) {
							name = k
							break
						}
					}
					if name != "" {
						fmt.Fprintf(&b, "%s  %04d  %-8s [%d] (%s)\n", indent, i, op, operand, name)
					} else {
						fmt.Fprintf(&b, "%s  %04d  %-8s [%d]\n", indent, i, op, operand)
					}
				default:
					fmt.Fprintf(&b, "%s  %04d  %-8s %d\n", indent, i, op, operand)
				}
				i += 2
			} else {
				fmt.Fprintf(&b, "%s  %04d  %s\n", indent, i, op)
				i++
			}
		}
	}
	for i, fn := range p.funcs {
		name := ""
		for k, v := range p.funcMap {
			if v == i {
				name = k
				break
			}
		}
		if name != "" {
			fmt.Fprintf(&b, "%sFunc [%d] %q:\n", indent, i, name)
		} else {
			fmt.Fprintf(&b, "%sFunc [%d]:\n", indent, i)
		}
		b.WriteString(fn.stringIndent(indent + "  "))
	}
	return b.String()
}

func (p *Program) String() string {
	return p.stringIndent("")
}

type Program struct {
	code []byte
	// TODO: refactor to 256 array as it only contains byte size
	cons []int
	funcs []*Program
	funcMap map[string]int
	paramCount int
	nextReg int
	symbols map[string]int
}

func Init() *Program {
	return &Program{
		cons: []int{},
		code: []byte{},
		symbols: map[string]int{},
		nextReg: 0,
		funcs: []*Program{},
		funcMap: map[string]int{},
	}
}

func (p *Program) allocReg() int {
	val := p.nextReg
	// ring buffer
	p.nextReg = (p.nextReg + 1) % 256
	return val
}


func (p *Program) Compile(stmts []Stmt) *Program {
	for _, stmt := range stmts {
		
		switch stmt := stmt.(type) {
		case *VarDecl:
			p.CompileExpr(stmt.Value)
			reg := p.allocReg()
			p.symbols[stmt.Name] = reg
			p.code = append(p.code, byte(OpStar), byte(reg))
		case *FuncDecl:
			idx := len(p.funcs)
			p.funcs = append(p.funcs, nil)
			p.funcMap[stmt.Name] = idx

			program := Init()
			program.paramCount = len(stmt.Params)
			program.funcMap = p.funcMap
			program.funcs = p.funcs

			for _, arg := range stmt.Params {
				reg := program.allocReg()
				program.symbols[arg] = reg
			}

			program.Compile(stmt.Body)
			p.funcs[idx] = program

		case *ReturnStmt:
			p.CompileExpr(stmt.Value)
			p.code = append(p.code, byte(OpReturn))
		case *ExprStmt:
			p.CompileExpr(stmt.Expr)
		}
	}	
	return p
}

// TODO: not sure if this should be the same register
func (p *Program) CompileExpr(expr Expr) {
	switch expr := expr.(type) {
	case *NumberLit:
		idx := len(p.cons)
		p.cons = append(p.cons, expr.Value)
		p.code = append(p.code, byte(OpLdaSmi), byte(idx))

	case *Ident:
		reg := p.symbols[expr.Name]
		p.code = append(p.code, byte(OpLdar), byte(reg))

	case *BinaryExpr:
		p.CompileExpr(expr.Left)	
		reg := p.allocReg()
		p.code = append(p.code, byte(OpStar), byte(reg))
		p.CompileExpr(expr.Right)
		
		switch expr.Op {
		case OADD:
			p.code = append(p.code, byte(OpAdd), byte(reg))
		case OMUL:
			p.code = append(p.code, byte(OpMul), byte(reg))
		}

	case *CallExpr:
		for _, arg := range expr.Args {
			p.CompileExpr(arg)	
			reg := p.allocReg()
			p.code = append(p.code, byte(OpStar), byte(reg))
		}

		p.code = append(p.code, byte(OpCall), byte(p.funcMap[expr.Func]))

	}
	// result is in accumulator after all operations anyway
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