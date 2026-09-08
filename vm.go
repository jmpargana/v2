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
	for vm.ip < len(program.code) {
		opcode := ByteCode(program.code[vm.ip])			
		vm.ip++
		
		switch opcode {
		case OpLdaSmi:
			vm.acc = int(program.cons[program.code[vm.ip]])
			vm.ip++
		case OpStar:
			idx := int(program.code[vm.ip])
			vm.ip++
			vm.reg[idx] = vm.acc
		case OpAdd:
			idx := int(program.code[vm.ip])
			vm.ip++
			vm.acc += vm.reg[idx]
		case OpMul:
			idx := int(program.code[vm.ip])
			vm.ip++
			vm.acc *= vm.reg[idx]
		}
	}
	return vm.acc
}