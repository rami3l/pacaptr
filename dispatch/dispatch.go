package dispatch

import (
	"fmt"

	"github.com/rami3l/pacapt-go/parser"
)

// Dispatch according to command line arguments
func Dispatch(args *parser.CmdArgs) (err error) {
	pacman := NewPacMan()
	kw := args.Keywords

	switch {
	case args.Query:
		switch {
		case args.C > 0:
			err = pacman.Qc(kw)
		case args.I:
			err = pacman.Qi(kw)
		case args.L:
			err = pacman.Ql(kw)
		case args.O:
			err = pacman.Qo(kw)
		case args.S:
			err = pacman.Qs(kw)
		case args.U:
			err = pacman.Qu(kw)
		default:
			err = pacman.Q(kw)
		}

	case args.Remove:
		switch {
		case args.S:
			err = pacman.Rs(kw)
		default:
			err = pacman.R(kw)
		}

	case args.Sync:
		switch {
		case args.C == 1:
			err = pacman.Sc(kw)
		case args.C == 2:
			err = pacman.Scc(kw)
		case args.C == 3:
			err = pacman.Sccc(kw)
		case args.I:
			err = pacman.Si(kw)
		case args.S:
			err = pacman.Ss(kw)
		case args.U && args.Y:
			err = pacman.Suy(kw)
		case args.U:
			err = pacman.Su(kw)
		case args.Y:
			err = pacman.Sy(kw)
		default:
			err = pacman.S(kw)
		}

	default:
		err = fmt.Errorf("pacapt: Feature not implemented")
	}

	return
}

// GetErrorCode for some error
// TODO: Make this function REALLY return correct error code
func GetErrorCode(_ error) int {
	return 1
}
