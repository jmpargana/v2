package main

import (
	"testing"
)

func intPtr(v int) *int {
	return &v
}

func TestLexer_Lex(t *testing.T) {
	tests := []struct {
		name string
		expr string
		want []Symbol
	}{
		{
			"single number",
			"123",
			[]Symbol{{Kind: SMI, IntVal: intPtr(123)}},
		},
		{
			"arithmetic expression",
			"123 + (23 * 43) * 3",
			[]Symbol{
				{Kind: SMI, IntVal: intPtr(123)},
				{Kind: ADD},
				{Kind: LPAR},
				{Kind: SMI, IntVal: intPtr(23)},
				{Kind: MUL},
				{Kind: SMI, IntVal: intPtr(43)},
				{Kind: RPAR},
				{Kind: MUL},
				{Kind: SMI, IntVal: intPtr(3)},
			},
		},
		{
			"variable declaration",
			"let x = 5;",
			[]Symbol{
				{Kind: LET},
				{Kind: IDENT_TOK, StrVal: "x"},
				{Kind: ASSIGN},
				{Kind: SMI, IntVal: intPtr(5)},
				{Kind: SEMI},
			},
		},
		{
			"function declaration",
			"function add(a, b) { return a + b; }",
			[]Symbol{
				{Kind: FUNCTION},
				{Kind: IDENT_TOK, StrVal: "add"},
				{Kind: LPAR},
				{Kind: IDENT_TOK, StrVal: "a"},
				{Kind: COMMA},
				{Kind: IDENT_TOK, StrVal: "b"},
				{Kind: RPAR},
				{Kind: LBRACE},
				{Kind: RETURN},
				{Kind: IDENT_TOK, StrVal: "a"},
				{Kind: ADD},
				{Kind: IDENT_TOK, StrVal: "b"},
				{Kind: SEMI},
				{Kind: RBRACE},
			},
		},
		{
			"const and var keywords",
			"const x = 1; var y = 2;",
			[]Symbol{
				{Kind: CONST},
				{Kind: IDENT_TOK, StrVal: "x"},
				{Kind: ASSIGN},
				{Kind: SMI, IntVal: intPtr(1)},
				{Kind: SEMI},
				{Kind: VAR},
				{Kind: IDENT_TOK, StrVal: "y"},
				{Kind: ASSIGN},
				{Kind: SMI, IntVal: intPtr(2)},
				{Kind: SEMI},
			},
		},
		{
			"identifier vs keyword",
			"let letters = 42;",
			[]Symbol{
				{Kind: LET},
				{Kind: IDENT_TOK, StrVal: "letters"},
				{Kind: ASSIGN},
				{Kind: SMI, IntVal: intPtr(42)},
				{Kind: SEMI},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var l Lexer
			got := l.Lex(tt.expr)
			if !SymbolsEqual(got, tt.want) {
				t.Errorf("Lex() = %v, want %v", got, tt.want)
			}
		})
	}
}
