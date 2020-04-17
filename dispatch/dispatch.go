package dispatch

import (
	"fmt"

	"github.com/rami3l/pacapt-ng/parser"
)

// Dispatch according to command line arguments.
func Dispatch(args *parser.CmdArgs) (err error) {
	pm := DetectPackManager(args)
	kws := args.Keywords

	switch {
	case args.Query:
		switch {
		case args.C == 1:
			err = pm.Qc(kws)
		case args.E:
			err = pm.Qe(kws)
		case args.I == 1:
			err = pm.Qi(kws)
		case args.K:
			err = pm.Qk(kws)
		case args.L:
			err = pm.Ql(kws)
		case args.M:
			err = pm.Qm(kws)
		case args.O:
			err = pm.Qo(kws)
		case args.P:
			err = pm.Qp(kws)
		case args.S:
			err = pm.Qs(kws)
		case args.U:
			err = pm.Qu(kws)
		default:
			err = pm.Q(kws)
		}

	case args.Remove:
		switch {
		case args.N && args.S:
			err = pm.Rns(kws)
		case args.N:
			err = pm.Rn(kws)
		case args.S:
			err = pm.Rs(kws)
		default:
			err = pm.R(kws)
		}

	case args.Sync:
		switch {
		case args.C == 1:
			err = pm.Sc(kws)
		case args.C == 2:
			err = pm.Scc(kws)
		case args.C == 3:
			err = pm.Sccc(kws)
		case args.G:
			err = pm.Sg(kws)
		case args.I == 1:
			err = pm.Si(kws)
		case args.I == 2:
			err = pm.Sii(kws)
		case args.L:
			err = pm.Sl(kws)
		case args.S:
			err = pm.Ss(kws)
		case args.U && args.Y:
			err = pm.Suy(kws)
		case args.U:
			err = pm.Su(kws)
		case args.Y:
			err = pm.Sy(kws)
		case args.W:
			err = pm.Sw(kws)
		default:
			err = pm.S(kws)
		}

	case args.Upgrade:
		err = pm.U(kws)

	default:
		err = fmt.Errorf("Invalid flag")
	}

	return
}

// GetErrorCode for some error.
// TODO: Make this function REALLY return correct error code
func GetErrorCode(_ error) int {
	return 1
}
