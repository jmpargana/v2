package main

import (
	"fmt"
	"slices"
	"unicode"
)

type Symbol struct {
	IntVal *int
	StrVal string
	Kind   SymbolKind
}

func (s Symbol) Equal(other Symbol) bool {
	if s.Kind != other.Kind {
		return false
	}
	if s.StrVal != other.StrVal {
		return false
	}
	if s.IntVal == nil || other.IntVal == nil {
		return s.IntVal == other.IntVal
	}
	return *s.IntVal == *other.IntVal
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
	LBRACE
	RBRACE
	ASSIGN
	COMMA
	SEMI
	IDENT_TOK
	LET
	CONST
	VAR
	FUNCTION
	RETURN
	EOF_TOK
)

func (k SymbolKind) String() string {
	switch k {
	case SMI:
		return "SMI"
	case ADD:
		return "+"
	case MUL:
		return "*"
	case LPAR:
		return "("
	case RPAR:
		return ")"
	case LBRACE:
		return "{"
	case RBRACE:
		return "}"
	case ASSIGN:
		return "="
	case COMMA:
		return ","
	case SEMI:
		return ";"
	case IDENT_TOK:
		return "IDENT"
	case LET:
		return "let"
	case CONST:
		return "const"
	case VAR:
		return "var"
	case FUNCTION:
		return "function"
	case RETURN:
		return "return"
	case EOF_TOK:
		return "EOF"
	default:
		return fmt.Sprintf("SymbolKind(%d)", k)
	}
}

var keywords = map[string]SymbolKind{
	"let":      LET,
	"const":    CONST,
	"var":      VAR,
	"function": FUNCTION,
	"return":   RETURN,
}

type Lexer struct{}

func (l *Lexer) Lex(src string) []Symbol {
	var res []Symbol
	i := 0
	for i < len(src) {
		ch := rune(src[i])

		if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
			i++
			continue
		}

		switch ch {
		case '+':
			res = append(res, Symbol{Kind: ADD})
			i++
		case '*':
			res = append(res, Symbol{Kind: MUL})
			i++
		case '(':
			res = append(res, Symbol{Kind: LPAR})
			i++
		case ')':
			res = append(res, Symbol{Kind: RPAR})
			i++
		case '{':
			res = append(res, Symbol{Kind: LBRACE})
			i++
		case '}':
			res = append(res, Symbol{Kind: RBRACE})
			i++
		case '=':
			res = append(res, Symbol{Kind: ASSIGN})
			i++
		case ',':
			res = append(res, Symbol{Kind: COMMA})
			i++
		case ';':
			res = append(res, Symbol{Kind: SEMI})
			i++
		default:
			if unicode.IsDigit(ch) {
				start := i
				for i < len(src) && unicode.IsDigit(rune(src[i])) {
					i++
				}
				n := 0
				for _, c := range src[start:i] {
					n = n*10 + int(c-'0')
				}
				res = append(res, Symbol{Kind: SMI, IntVal: &n})
			} else if unicode.IsLetter(ch) || ch == '_' {
				start := i
				for i < len(src) && (unicode.IsLetter(rune(src[i])) || unicode.IsDigit(rune(src[i])) || src[i] == '_') {
					i++
				}
				word := src[start:i]
				if kind, ok := keywords[word]; ok {
					res = append(res, Symbol{Kind: kind})
				} else {
					res = append(res, Symbol{Kind: IDENT_TOK, StrVal: word})
				}
			} else {
				panic(fmt.Sprintf("unexpected character: %c", ch))
			}
		}
	}
	return res
}
