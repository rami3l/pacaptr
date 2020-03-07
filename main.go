package main

import (
	"fmt"
	"os"

	"github.com/rami3l/pacapt-go/dispatch"
	"github.com/rami3l/pacapt-go/parser"
)

func main() {
	args, err := parser.Run()
	if err != nil {
		fmt.Println(err)
		os.Exit(dispatch.GetErrorCode(err))
	}

	err = dispatch.Dispatch(args)
	if err != nil {
		fmt.Println(err)
		os.Exit(dispatch.GetErrorCode(err))
	}

	return
}
