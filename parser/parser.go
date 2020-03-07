package parser

import (
	"fmt"
	"os"

	"github.com/akamensky/argparse"
)

// CmdArgs collects pacapt arguments.
type CmdArgs struct {
	Query, Remove, Sync bool
	I, L, O, S, U, Y    bool
	C                   int
}

// Run the argument parser
func Run() (args *CmdArgs, err error) {
	// Create new parser object
	parser := argparse.NewParser("pacapt", "A pacman-like wrapper for package managers")

	// Operations include Q(uery), R(emove), and S(ync).
	query := parser.Flag("Q", "Query", &argparse.Options{Help: "Query"})
	remove := parser.Flag("R", "Remove", &argparse.Options{Help: "Remove"})
	sync := parser.Flag("S", "Sync", &argparse.Options{Help: "Sync"})

	// Flags
	// ! WARNING
	// ! Some long flag names are completely different for different operations,
	// ! but I think mose of us just use the shorthand form anyway...
	// see: https://www.archlinux.org/pacman/pacman.8.html#
	i := parser.Flag("i", "info", &argparse.Options{Help: "(-Q/S) info"})
	l := parser.Flag("l", "list", &argparse.Options{Help: "(-Q) list"})
	o := parser.Flag("o", "owns", &argparse.Options{Help: "(-Q) owns"})
	s := parser.Flag("s", "search", &argparse.Options{Help: "(-S) search | (-R) recursive"})
	u := parser.Flag("u", "sysupgrade", &argparse.Options{Help: "(-S) sysupgrade"})
	y := parser.Flag("y", "refresh", &argparse.Options{Help: "(-S) refresh"})

	// Flagcounters
	c := parser.FlagCounter("c", "clean", &argparse.Options{Help: "(-S) clean"})

	// Parse input
	err = parser.Parse(os.Args)

	// A naive implementation of a mutually exclusive check.
	count := 0
	for _, op := range []bool{*query, *remove, *sync} {
		if op {
			count++
		}
	}
	if count != 1 {
		err = fmt.Errorf("pacapt: exactly ONE operation expected, found %d", count)
		return
	}

	// Collect arguments
	args = &CmdArgs{
		*query, *remove, *sync,
		*i, *l, *o, *s, *u, *y,
		*c,
	}

	return
}
