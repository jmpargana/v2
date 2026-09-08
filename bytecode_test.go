package main

import (
	"testing"
)

func TestProgram_Compile(t *testing.T) {
	tests := []struct {
		name string // description of this test case
		// Named input parameters for target function.
		ast  *AST
		want *Program
	}{
		{
			"1 * (2 + 3)",
			&AST{
				Left: &AST{Value: intPtr(1)},
				Op: OMUL,
				Right: &AST{
					Left: &AST{Value: intPtr(2)},
					Op: OADD,
					Right: &AST{Value: intPtr(3)},
				},
			},
			&Program{
				code: []byte{
					byte(OpLdaSmi),
					byte(0),
					byte(OpStar),
					byte(0),
					byte(OpLdaSmi),
					byte(1),
					byte(OpStar),
					byte(1),
					byte(OpLdaSmi),
					byte(2),
					byte(OpAdd),
					byte(1),
					byte(OpMul),
					byte(0),
				},
				cons: []int{1, 2, 3},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// TODO: construct the receiver type.
			var p Program
			got := p.Compile(tt.ast)
			if !got.Equals(tt.want) {
				t.Errorf("Compile() mismatch for AST: %s\ngot:\n%s\nwant:\n%s", tt.ast, got, tt.want)
			}
		})
	}
}
