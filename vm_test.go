package main

import (
	"testing"
)

func TestVM_FDE(t *testing.T) {
	tests := []struct {
		name string // description of this test case
		// Named input parameters for target function.
		program Program
		want    int
	}{
		{
			"first example",
			Program{
				code: []byte{
					byte(OpPush),
					byte(0),
					byte(OpPush),
					byte(1),
					byte(OpPush),
					byte(2),
					byte(OpMul),
					byte(OpAdd),
				},
				cons: []int{30, 20, 40},
			},
			830,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			vm := Start()
			got := vm.FDE(tt.program)
			// TODO: update the condition below to compare got with tt.want.
			if got != tt.want {
				t.Errorf("FDE() = %v, want %v", got, tt.want)
			}
		})
	}
}
