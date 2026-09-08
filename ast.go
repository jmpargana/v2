package main

import (
	"fmt"
	"strings"
)

type Op int

const (
	OADD Op = iota
	OMUL
)

func (op Op) String() string {
	switch op {
	case OADD:
		return "+"
	case OMUL:
		return "*"
	default:
		return fmt.Sprintf("Op(%d)", op)
	}
}

type Node interface {
	node()
	String() string
}

type Expr interface {
	Node
	expr()
}

type Stmt interface {
	Node
	stmt()
}

// --- Expressions ---

type NumberLit struct {
	Value int
}

func (*NumberLit) node() {}
func (*NumberLit) expr() {}
func (n *NumberLit) String() string {
	return fmt.Sprintf("%d", n.Value)
}

type Ident struct {
	Name string
}

func (*Ident) node() {}
func (*Ident) expr() {}
func (id *Ident) String() string {
	return id.Name
}

type BinaryExpr struct {
	Op          Op
	Left, Right Expr
}

func (*BinaryExpr) node() {}
func (*BinaryExpr) expr() {}
func (b *BinaryExpr) String() string {
	return fmt.Sprintf("(%s %s %s)", b.Left, b.Op, b.Right)
}

type CallExpr struct {
	Func string
	Args []Expr
}

func (*CallExpr) node() {}
func (*CallExpr) expr() {}
func (c *CallExpr) String() string {
	args := make([]string, len(c.Args))
	for i, a := range c.Args {
		args[i] = a.String()
	}
	return fmt.Sprintf("%s(%s)", c.Func, strings.Join(args, ", "))
}

// --- Statements ---

type VarDecl struct {
	Kind  SymbolKind
	Name  string
	Value Expr
}

func (*VarDecl) node() {}
func (*VarDecl) stmt() {}
func (v *VarDecl) String() string {
	return fmt.Sprintf("%s %s = %s;", v.Kind, v.Name, v.Value)
}

type FuncDecl struct {
	Name   string
	Params []string
	Body   []Stmt
}

func (*FuncDecl) node() {}
func (*FuncDecl) stmt() {}
func (f *FuncDecl) String() string {
	var b strings.Builder
	fmt.Fprintf(&b, "function %s(%s) {", f.Name, strings.Join(f.Params, ", "))
	for _, s := range f.Body {
		fmt.Fprintf(&b, " %s", s)
	}
	b.WriteString(" }")
	return b.String()
}

type ReturnStmt struct {
	Value Expr
}

func (*ReturnStmt) node() {}
func (*ReturnStmt) stmt() {}
func (r *ReturnStmt) String() string {
	return fmt.Sprintf("return %s;", r.Value)
}

type ExprStmt struct {
	Expr Expr
}

func (*ExprStmt) node() {}
func (*ExprStmt) stmt() {}
func (e *ExprStmt) String() string {
	return fmt.Sprintf("%s;", e.Expr)
}
