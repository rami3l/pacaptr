package dispatch

import (
	"fmt"

	"github.com/rami3l/pacapt-go/parser"
)

// Dispatch according to command line arguments.
func Dispatch(args *parser.CmdArgs) (err error) {
	pm := DetectPackManager(args.DryRun, args.NoConfirm)
	kw := args.Keywords

	switch {
	case args.Query:
		switch {
		case args.C == 1:
			err = pm.Qc(kw)
		case args.E:
			err = pm.Qe(kw)
		case args.I == 1:
			err = pm.Qi(kw)
		case args.K:
			err = pm.Qk(kw)
		case args.L:
			err = pm.Ql(kw)
		case args.M:
			err = pm.Qm(kw)
		case args.O:
			err = pm.Qo(kw)
		case args.P:
			err = pm.Qp(kw)
		case args.S:
			err = pm.Qs(kw)
		case args.U:
			err = pm.Qu(kw)
		default:
			err = pm.Q(kw)
		}

	case args.Remove:
		switch {
		case args.N && args.S:
			err = pm.Rns(kw)
		case args.N:
			err = pm.Rn(kw)
		case args.S:
			err = pm.Rs(kw)
		default:
			err = pm.R(kw)
		}

	case args.Sync:
		switch {
		case args.C == 1:
			err = pm.Sc(kw)
		case args.C == 2:
			err = pm.Scc(kw)
		case args.C == 3:
			err = pm.Sccc(kw)
		case args.G:
			err = pm.Sg(kw)
		case args.I == 1:
			err = pm.Si(kw)
		case args.I == 2:
			err = pm.Sii(kw)
		case args.L:
			err = pm.Sl(kw)
		case args.S:
			err = pm.Ss(kw)
		case args.U && args.Y:
			err = pm.Suy(kw)
		case args.U:
			err = pm.Su(kw)
		case args.Y:
			err = pm.Sy(kw)
		case args.W:
			err = pm.Sw(kw)
		default:
			err = pm.S(kw)
		}

	case args.Upgrade:
		err = pm.U(kw)

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
