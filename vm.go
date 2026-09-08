package main

type VM struct {
	ip int // index to slice
	stack []int
}

func Start() *VM {
	return &VM{
		ip: 0,
		stack: []int{},
	}
}

func (vm *VM) Pop() int {
	n := len(vm.stack) - 1
	val := vm.stack[n]
	vm.stack = vm.stack[:n]
	return val
}

func (vm *VM) Push(el int) {
	vm.stack = append(vm.stack, el)
}

// Fetch, Decode, Execute
func (vm *VM) FDE(program Program) int {
	for vm.ip < len(program.code) {
		opcode := ByteCode(program.code[vm.ip])			
		vm.ip++
		
		switch opcode {
		case OpPush:
			idx := int(program.code[vm.ip])
			vm.ip++
			vm.Push(program.cons[idx])
		case OpMul:
			b, a := vm.Pop(), vm.Pop()
			vm.Push(a * b)	
		case OpAdd:
			b, a := vm.Pop(), vm.Pop()
			vm.Push(a + b)	
		}
	}
	return vm.Pop()
}