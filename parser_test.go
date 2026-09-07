package main

import "testing"

func TestParser_parseExpr(t *testing.T) {
	tests := []struct {
		name string // description of this test case
		given []Symbol
		want *AST
	}{
		{
			"simple expression",
			[]Symbol{
				{Kind: SMI, Value: intPtr(30)},
			},
			&AST{
				Value: intPtr(30),
			},
		},
		{
			"complex expression",
			[]Symbol{
				{Kind: SMI, Value: intPtr(30)},
				{Kind: ADD},
				{Kind: LPAR},
				{Kind: SMI, Value: intPtr(20)},
				{Kind: MUL},
				{Kind: SMI, Value: intPtr(40)},
				{Kind: RPAR},
			},
			&AST{
				Op: OADD,
				Left: &AST{Value: intPtr(30)},
				Right: &AST{
					Op: OMUL,
					Left: &AST{
						Value: intPtr(20),
					},
					Right: &AST{
						Value: intPtr(40),
					},
				},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// TODO: construct the receiver type.
			p := New(tt.given)
			got := p.parseExpr()
			// TODO: update the condition below to compare got with tt.want.
			if !tt.want.Equals(got) {
				t.Errorf("parseExpr() = %v, want %v", got, tt.want)
			}
		})
	}
}
