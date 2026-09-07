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
				code: []ByteCode{
					OpPush,
					OpPush,
					OpPush,
					OpAdd,
					OpMul,
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
			// TODO: update the condition below to compare got with tt.want.
			if !got.Equals(tt.want) {
				t.Errorf("Compile() = %v, want %v", got, tt.want)
			}
		})
	}
}
