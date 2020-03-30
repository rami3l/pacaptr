package parser

import (
	"fmt"
	"os"
	"strings"

	"github.com/akamensky/argparse"
)

// TODO: Support more operation combinations.

// CmdArgs collects pacapt arguments.
type CmdArgs struct {
	Query, Remove, Sync, Upgrade       bool
	E, G, K, L, M, N, O, P, S, U, W, Y bool
	C, I                               int
	Keywords                           []string
	DryRun                             bool
	NoConfirm                          bool
}

// stripKeywords distinguishes between pacapt flags and package names.
// ! WARNING: Extremely dirty...
func stripKeywords(args []string) (cmd []string, kw []string) {
	cmd = args[:1]

	for _, s := range args[1:] {
		if strings.HasPrefix(s, "-") {
			cmd = append(cmd, s)
		} else {
			kw = append(kw, s)
		}
	}

	// fmt.Printf("cmd: %s, kw: %s\n", cmd, keywords)
	return
}

// Run the argument parser.
func Run() (args *CmdArgs, err error) {
	// Create new parser object
	parser := argparse.NewParser("pacapt", "A pacman-like wrapper for package managers")

	// Operations include Q(uery), R(emove), and S(ync).
	query := parser.Flag("Q", "Query", &argparse.Options{Help: "Query"})
	remove := parser.Flag("R", "Remove", &argparse.Options{Help: "Remove"})
	sync := parser.Flag("S", "Sync", &argparse.Options{Help: "Sync"})
	upgrade := parser.Flag("U", "Upgrade", &argparse.Options{Help: "Upgrade"})

	// Flags
	// ! WARNING
	// ! Some long flag names are completely different for different operations,
	// ! but I think mose of us just use the shorthand form anyway...
	// see: https://www.archlinux.org/pacman/pacman.8.html

	e := parser.Flag("e", "explicit", &argparse.Options{Help: "(-Q) explicit"})
	g := parser.Flag("g", "groups", &argparse.Options{Help: "(-Q/S) groups"})
	k := parser.Flag("k", "check", &argparse.Options{Help: "(-Q) check"})
	l := parser.Flag("l", "list", &argparse.Options{Help: "(-Q) list"})
	m := parser.Flag("m", "foreign", &argparse.Options{Help: "(-Q) foreign"})
	n := parser.Flag("n", "nosave", &argparse.Options{Help: "(-R) nosave"})
	o := parser.Flag("o", "owns", &argparse.Options{Help: "(-Q) owns"})
	p := parser.Flag("p", "print", &argparse.Options{Help: "(-Q/R/S) print"})
	s := parser.Flag("s", "search", &argparse.Options{Help: "(-S) search | (-R) recursive"})
	u := parser.Flag("u", "sysupgrade", &argparse.Options{Help: "(-S) sysupgrade"})
	w := parser.Flag("w", "downloadonly", &argparse.Options{Help: "(-S) downloadonly"})
	y := parser.Flag("y", "refresh", &argparse.Options{Help: "(-S) refresh"})

	// Flagcounters
	c := parser.FlagCounter("c", "clean", &argparse.Options{Help: "(-S) clean"})
	i := parser.FlagCounter("i", "info", &argparse.Options{Help: "(-Q/S) info"})

	// Other flags
	dryRun := parser.Flag("", "dryrun", &argparse.Options{Help: "Perform a dry run"})
	dryRunAlt := parser.Flag("", "dry-run", &argparse.Options{Help: "Perform a dry run"})
	noConfirm := parser.Flag("", "noconfirm", &argparse.Options{Help: "Answer yes to every question"})

	// Parse input
	cmd, kw := stripKeywords(os.Args)
	if err = parser.Parse(cmd); err != nil {
		return
	}

	// A naive implementation of a mutually exclusive check.
	count := 0
	for _, op := range []bool{*query, *remove, *sync, *upgrade} {
		if op {
			count++
		}
	}
	if count != 1 {
		err = fmt.Errorf("pacapt: Exactly 1 operation expected, found %d", count)
		return
	}

	// Collect arguments
	args = &CmdArgs{
		*query, *remove, *sync, *upgrade,
		*e, *g, *k, *l, *m, *n, *o, *p, *s, *u, *w, *y,
		*c, *i,
		kw,
		*dryRun || *dryRunAlt,
		*noConfirm,
	}

	return
}
