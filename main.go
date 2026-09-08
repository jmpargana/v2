package main

import (
	"fmt"
	"os"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintln(os.Stderr, "usage: g8 <file.js>")
		os.Exit(1)
	}

	src, err := os.ReadFile(os.Args[1])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	tokens := (&Lexer{}).Lex(string(src))
	stmts := New(tokens).Parse()
	program := Init().Compile(stmts)
	result := Start().FDE(*program)

	fmt.Println(result)
}
