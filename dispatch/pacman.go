package dispatch

// PacMan represents a PACkage MANager
type PacMan interface {
	Q([]string) error
	Qc([]string) error
	Qe([]string) error
	Qi([]string) error
	Qk([]string) error
	Ql([]string) error
	Qm([]string) error
	Qo([]string) error
	Qp([]string) error
	Qs([]string) error
	Qu([]string) error
	R([]string) error
	Rn([]string) error
	Rns([]string) error
	Rs([]string) error
	S([]string) error
	Sc([]string) error
	Scc([]string) error
	Sccc([]string) error
	Sg([]string) error
	Si([]string) error
	Sii([]string) error
	Sl([]string) error
	Ss([]string) error
	Su([]string) error
	Suy([]string) error
	Sw([]string) error
	Sy([]string) error
	U([]string) error
}

// NewPacMan detects the package manager in use
// TODO: Make this function REALLY detect package managers
func NewPacMan() PacMan {
	return &Homebrew{}
}
