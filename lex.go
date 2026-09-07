package main

import (
	"fmt"
	"slices"
	"strconv"
)


type Symbol struct {
	Value *int
	Kind SymbolKind
}

func (s Symbol) Equal(other Symbol) bool {
	if s.Kind != other.Kind {
		return false
	}

	if s.Value == nil || other.Value == nil {
		return s.Value == other.Value
	}

	return *s.Value == *other.Value
}

func SymbolsEqual(a, b []Symbol) bool {
	return slices.EqualFunc(a, b, Symbol.Equal)
}

type SymbolKind int

const (
	SMI SymbolKind = iota
	ADD
	MUL
	LPAR
	RPAR
)

type Lexer struct {}

func (l *Lexer) Lex(expr string) []Symbol {
	var res []Symbol
	var stack string
	for i := 0; i<len(expr) ;i++ {
		switch expr[i] {
		case '+':
			if len(stack) > 0 {
				n, _ := strconv.Atoi(stack)
				res = append(res, Symbol{
					Kind: SMI,
					Value: &n,
				})
				stack = ""
			}
			res = append(res, Symbol{
				Kind: ADD,
			})
		case '*':
			if len(stack) > 0 {
				n, _ := strconv.Atoi(stack)
				res = append(res, Symbol{
					Kind: SMI,
					Value: &n,
				})
				stack = ""
			}
			res = append(res, Symbol{
				Kind: MUL,
			})
		case '(':
			res = append(res, Symbol{
				Kind: LPAR,
			})
		case ')':
			if len(stack) > 0 {
				n, _ := strconv.Atoi(stack)
				res = append(res, Symbol{
					Kind: SMI,
					Value: &n,
				})
			}
			res = append(res, Symbol{
				Kind: RPAR,
			})
			stack = ""
		case ' ':
			continue
		default:
			// must be number
			stack = fmt.Sprintf("%s%c", stack, expr[i])
		}
	}
	if len(stack) > 0 {
		n, _ := strconv.Atoi(stack)
		res = append(res, Symbol{
			Kind: SMI,
			Value: &n,
		})
	}
	return res
}
