package main

import (
	"os"

	"github.com/fatih/color"

	"github.com/rami3l/pacaptr/dispatch"
	"github.com/rami3l/pacaptr/parser"
)

func main() {
	red := color.New(color.FgRed)

	args, err := parser.Run()
	if err != nil {
		red.Fprintf(os.Stderr, ":: parser: %s\n", err)
		os.Exit(dispatch.GetErrorCode(err))
	}

	err = dispatch.Dispatch(args)
	if err != nil {
		// red.Fprintf(os.Stderr, ":: %s\n", err)
		os.Exit(dispatch.GetErrorCode(err))
	}

	return
}
