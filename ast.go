package main

/**
Language:

a + (b * c)

*/
type Op int

const (
	OADD Op = iota
	OMUL
)

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