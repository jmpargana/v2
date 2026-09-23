package main

import "fmt"

type VM struct {
	acc int
	stack []*Frame
}

func (vm *VM) String() string {
	return fmt.Sprintf("VM{acc: %d, frames: %d}", vm.acc, len(vm.stack))
}

type Frame struct {
	ip int
	reg [256]int
	closure Value
}

func Start() *VM {
	return &VM{
		acc: 0,
		stack: []*Frame{},
	}
}

func (vm *VM) FDE(program Program) int {
	vm.stack = append(vm.stack, &Frame{
		program: &program,
		reg:     [256]int{},
		ip:      0,
	})

	for len(vm.stack) > 0 {
		f := vm.stack[len(vm.stack)-1]
		if f.ip >= len(f.program.code) {
			vm.stack = vm.stack[:len(vm.stack)-1]
			continue
		}

		opcode := ByteCode(f.program.code[f.ip])
		f.ip++

		switch opcode {
		case OpLdaSmi:
			vm.acc = f.program.cons[f.program.code[f.ip]]
			f.ip++
		case OpStar:
			idx := f.program.code[f.ip]
			f.ip++
			f.reg[idx] = vm.acc
		case OpLdar:
			idx := f.program.code[f.ip]
			f.ip++
			vm.acc = f.reg[idx]
		case OpAdd:
			idx := f.program.code[f.ip]
			f.ip++
			vm.acc += f.reg[idx]
		case OpMul:
			idx := f.program.code[f.ip]
			f.ip++
			vm.acc *= f.reg[idx]
		case OpCall:
			funcIdx := f.program.code[f.ip]
			f.ip++
			function := f.program.funcs[funcIdx]
			callFrame := &Frame{
				program: function,
				reg:     [256]int{},
				ip:      0,
			}
			for i := range function.paramCount {
				callFrame.reg[i] = f.reg[f.program.nextReg-function.paramCount+i]
			}
			vm.stack = append(vm.stack, callFrame)
		case OpReturn:
			vm.stack = vm.stack[:len(vm.stack)-1]
		}
	}

	return vm.acc
}