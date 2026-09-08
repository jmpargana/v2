package main

import (
	"reflect"
	"testing"
)

func TestParser_Parse(t *testing.T) {
	tests := []struct {
		name  string
		given []Symbol
		want  []Stmt
	}{
		{
			"expression statement",
			[]Symbol{
				{Kind: SMI, IntVal: intPtr(30)},
				{Kind: SEMI},
			},
			[]Stmt{
				&ExprStmt{Expr: &NumberLit{Value: 30}},
			},
		},
		{
			"binary expression with precedence",
			[]Symbol{
				{Kind: SMI, IntVal: intPtr(1)},
				{Kind: ADD},
				{Kind: SMI, IntVal: intPtr(2)},
				{Kind: MUL},
				{Kind: SMI, IntVal: intPtr(3)},
				{Kind: SEMI},
			},
			[]Stmt{
				&ExprStmt{Expr: &BinaryExpr{
					Op:   OADD,
					Left: &NumberLit{Value: 1},
					Right: &BinaryExpr{
						Op:    OMUL,
						Left:  &NumberLit{Value: 2},
						Right: &NumberLit{Value: 3},
					},
				}},
			},
		},
		{
			"parenthesized expression",
			[]Symbol{
				{Kind: SMI, IntVal: intPtr(30)},
				{Kind: ADD},
				{Kind: LPAR},
				{Kind: SMI, IntVal: intPtr(20)},
				{Kind: MUL},
				{Kind: SMI, IntVal: intPtr(40)},
				{Kind: RPAR},
				{Kind: SEMI},
			},
			[]Stmt{
				&ExprStmt{Expr: &BinaryExpr{
					Op:   OADD,
					Left: &NumberLit{Value: 30},
					Right: &BinaryExpr{
						Op:    OMUL,
						Left:  &NumberLit{Value: 20},
						Right: &NumberLit{Value: 40},
					},
				}},
			},
		},
		{
			"variable declaration",
			[]Symbol{
				{Kind: LET},
				{Kind: IDENT_TOK, StrVal: "x"},
				{Kind: ASSIGN},
				{Kind: SMI, IntVal: intPtr(5)},
				{Kind: ADD},
				{Kind: SMI, IntVal: intPtr(3)},
				{Kind: SEMI},
			},
			[]Stmt{
				&VarDecl{
					Kind:  LET,
					Name:  "x",
					Value: &BinaryExpr{Op: OADD, Left: &NumberLit{Value: 5}, Right: &NumberLit{Value: 3}},
				},
			},
		},
		{
			"function declaration with return",
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
			[]Stmt{
				&FuncDecl{
					Name:   "add",
					Params: []string{"a", "b"},
					Body: []Stmt{
						&ReturnStmt{
							Value: &BinaryExpr{
								Op:    OADD,
								Left:  &Ident{Name: "a"},
								Right: &Ident{Name: "b"},
							},
						},
					},
				},
			},
		},
		{
			"function call",
			[]Symbol{
				{Kind: IDENT_TOK, StrVal: "add"},
				{Kind: LPAR},
				{Kind: SMI, IntVal: intPtr(1)},
				{Kind: COMMA},
				{Kind: SMI, IntVal: intPtr(2)},
				{Kind: RPAR},
				{Kind: SEMI},
			},
			[]Stmt{
				&ExprStmt{Expr: &CallExpr{
					Func: "add",
					Args: []Expr{&NumberLit{Value: 1}, &NumberLit{Value: 2}},
				}},
			},
		},
		{
			"no-arg function",
			[]Symbol{
				{Kind: FUNCTION},
				{Kind: IDENT_TOK, StrVal: "noop"},
				{Kind: LPAR},
				{Kind: RPAR},
				{Kind: LBRACE},
				{Kind: RBRACE},
			},
			[]Stmt{
				&FuncDecl{
					Name:   "noop",
					Params: nil,
					Body:   nil,
				},
			},
		},
		{
			"multi-statement program",
			[]Symbol{
				{Kind: LET},
				{Kind: IDENT_TOK, StrVal: "x"},
				{Kind: ASSIGN},
				{Kind: SMI, IntVal: intPtr(10)},
				{Kind: SEMI},
				{Kind: LET},
				{Kind: IDENT_TOK, StrVal: "y"},
				{Kind: ASSIGN},
				{Kind: IDENT_TOK, StrVal: "x"},
				{Kind: MUL},
				{Kind: SMI, IntVal: intPtr(2)},
				{Kind: SEMI},
			},
			[]Stmt{
				&VarDecl{Kind: LET, Name: "x", Value: &NumberLit{Value: 10}},
				&VarDecl{Kind: LET, Name: "y", Value: &BinaryExpr{
					Op:    OMUL,
					Left:  &Ident{Name: "x"},
					Right: &NumberLit{Value: 2},
				}},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p := New(tt.given)
			got := p.Parse()
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("Parse():\ngot:  %v\nwant: %v", got, tt.want)
			}
		})
	}
}
