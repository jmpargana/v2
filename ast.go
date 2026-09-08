package main

import (
	"fmt"
	"strconv"
)

/**
Language:

a + (b * c)

*/
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

func (a *AST) String() string {
	if a == nil {
		return "<nil>"
	}
	if a.Value != nil {
		return strconv.Itoa(*a.Value)
	}
	return fmt.Sprintf("(%s %s %s)", a.Left, a.Op, a.Right)
}

type AST struct {
	Op Op
	Left, Right *AST
	Value *int
}

func (a *AST) Equals(b *AST) bool {
	if a == nil || b == nil {
		return a == b
	}

	if a.Value != nil || b.Value != nil {
		if a.Value == nil || b.Value == nil {
			return false
		}
		return *a.Value == *b.Value
	}

	return a.Op == b.Op &&
		a.Left.Equals(b.Left) &&
		a.Right.Equals(b.Right)
}