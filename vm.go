package main

import (
	"fmt"
	"strings"
)

type VM struct {
	ip int // index to slice
	acc int
	reg [256]int
}

func Start() *VM {
	return &VM{
		ip: 0,
		acc: 0,
		reg: [256]int{},
	}
}

func (vm *VM) String() string {
	var b strings.Builder
	fmt.Fprintf(&b, "IP:  %d\nACC: %d\n", vm.ip, vm.acc)
	for i, v := range vm.reg {
		if v != 0 {
			fmt.Fprintf(&b, "r%-3d %d\n", i, v)
		}
	}
	return b.String()
}

// Fetch, Decode, Execute
func (vm *VM) FDE(program Program) int {
	return 0
	// for vm.ip < len(program.code) {
	// 	opcode := ByteCode(program.code[vm.ip])			
	// 	vm.ip++
		
	// 	switch opcode {
	// 	case OpPush:
	// 		idx := int(program.code[vm.ip])
	// 		vm.ip++
	// 		vm.Push(program.cons[idx])
	// 	case OpMul:
	// 		b, a := vm.Pop(), vm.Pop()
	// 		vm.Push(a * b)	
	// 	case OpAdd:
	// 		b, a := vm.Pop(), vm.Pop()
	// 		vm.Push(a + b)	
	// 	}
	// }
	// return vm.Pop()
}