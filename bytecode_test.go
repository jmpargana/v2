package main

import "testing"


func TestProgram_Compile(t *testing.T) {
	tests := []struct {
		name string // description of this test case
		// Named input parameters for target function.
		stmts []Stmt
		want  *Program
	}{
		{
			"example from lesson",
			[]Stmt{
				&VarDecl{
					Kind: LET,
					Name: "x",
					Value: &NumberLit{Value: 10},
				},
				&VarDecl{
					Kind: LET,
					Name: "y",
					Value: &BinaryExpr{
						Op: OADD,
						Left: &NumberLit{Value: 20},
						Right: &Ident{Name: "x"},
					},
				},
				&ExprStmt{
					Expr: &BinaryExpr{
						Op: OMUL,
						Left: &Ident{Name: "x"},
						Right: &Ident{Name: "y"},
					},
				},
			},
			&Program{
				cons: []int{10, 20},
				code: []byte{
					byte(OpLdaSmi),
					byte(0),
					byte(OpStar),
					byte(0),
					byte(OpLdaSmi),
					byte(1),
					byte(OpStar),
					byte(1),
					byte(OpLdar),
					byte(0),
					byte(OpAdd),
					byte(1),
					byte(OpStar),
					byte(2),
					byte(OpLdar),
					byte(0),
					byte(OpStar),
					byte(3),
					byte(OpLdar),
					byte(2),
					byte(OpMul),
					byte(3),
				},
				nextReg: 4,
				symbols: map[string]int{
					"x": 0,
					"y": 2,
				},
				funcs: []*Program{},
				funcMap: map[string]int{},
			},
		},
		{
			"add caller and callee",
			[]Stmt{
				&FuncDecl{
					Name: "add",
					Params: []string{"a", "b"},
					Body: []Stmt{
						&ReturnStmt{
							Value: &BinaryExpr{
								Op: OADD,
								Left: &Ident{Name: "a"},
								Right: &Ident{Name: "b"},
							},
						},
					},
				},
				&ExprStmt{
					Expr: &CallExpr{
						Func: "add",
						Args: []Expr{
							&NumberLit{Value: 10},
							&NumberLit{Value: 20},
						},
					},
				},
			},
			&Program{
				cons: []int{10, 20},
				code: []byte{
					byte(OpLdaSmi),
					byte(0),
					byte(OpStar),
					byte(0),
					byte(OpLdaSmi),
					byte(1),
					byte(OpStar),
					byte(1),
					byte(OpCall),
					byte(0),
				},
				nextReg: 2,
				symbols: map[string]int{},
				paramCount: 0,
				funcs: []*Program{
					{
						cons: []int{},
						code: []byte{
							byte(OpLdar),
							byte(0),
							byte(OpStar),
							byte(2),
							byte(OpLdar),
							byte(1),
							byte(OpAdd),
							byte(2),
							byte(OpReturn),
						},
						paramCount: 2,
						nextReg: 3,
						symbols: map[string]int{
							"a": 0,
							"b": 1,
						},
						funcs: []*Program{},
						funcMap: map[string]int{},
					},
				},
				funcMap: map[string]int{"add": 0},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// TODO: construct the receiver type.
			p := Init()
			got := p.Compile(tt.stmts)
			// TODO: update the condition below to compare got with tt.want.
			if !got.Equals(tt.want) {
				t.Errorf("Compile() = %v, want %v", got, tt.want)
			}
		})
	}
}
