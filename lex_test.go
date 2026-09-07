package main

import (
	"testing"
)

func intPtr(v int) *int {
	return &v
}

func TestLexer_Lex(t *testing.T) {
	tests := []struct {
		name string // description of this test case
		// Named input parameters for target function.
		expr string
		want []Symbol
	}{
		{
			"single digit",
			"123",
			[]Symbol{{Kind: SMI, Value: intPtr(123)}},
		},
		{
			"complex expression",
			"123 + (23 * 43) * 3",
			[]Symbol{
				{Kind: SMI, Value: intPtr(123)},
				{Kind: ADD},
				{Kind: LPAR},
				{Kind: SMI, Value: intPtr(23)},
				{Kind: MUL},
				{Kind: SMI, Value: intPtr(43)},
				{Kind: RPAR},
				{Kind: MUL},
				{Kind: SMI, Value: intPtr(3)},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// TODO: construct the receiver type.
			var l Lexer
			got := l.Lex(tt.expr)
			// TODO: update the condition below to compare got with tt.want.
			if !SymbolsEqual(got, tt.want) {
				t.Errorf("Lex() = %v, want %v", got, tt.want)
			}
		})
	}
}
