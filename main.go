package main

import (
	"fmt"

	"github.com/rami3l/pacapt-go/parser"
)

func main() {
	fmt.Println("Hello, pacapt!")
	args, err := parser.Run()
	if err != nil {
		fmt.Println(err)
	} else {
		fmt.Println(args)
	}
}
