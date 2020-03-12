package dispatch

import (
	"fmt"
	"os/exec"
)

// Homebrew package manager config
type Homebrew struct{}

// For method implementation see: https://golang.org/src/os/exec/example_test.go
func (self *Homebrew) Q(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Qc(keywords []string) (err error) {
	out, err := exec.Command("brew", append([]string{"log"}, keywords...)...).Output()
	if err != nil {
		return
	}
	fmt.Printf("%s\n", out)
	return
}

func (self *Homebrew) Qe(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Qi(keywords []string) (err error) {
	out, err := exec.Command("brew", append([]string{"info"}, keywords...)...).Output()
	if err != nil {
		return
	}
	fmt.Printf("%s\n", out)
	return
}

func (self *Homebrew) Qk(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Ql(keywords []string) (err error) {
	out, err := exec.Command("brew", append([]string{"list"}, keywords...)...).Output()
	if err != nil {
		return
	}
	fmt.Printf("%s\n", out)
	return
}

func (self *Homebrew) Qm(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Qo(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Qp(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Qs(keywords []string) (err error) {
	// TODO: it seems that the output of `brew list python` in fish has a mechanism against duplication:
	// /usr/local/Cellar/python/3.6.0/Frameworks/Python.framework/ (1234 files)
	out, err := exec.Command("brew", append([]string{"list"}, keywords...)...).Output()
	if err != nil {
		return
	}
	fmt.Printf("%s\n", out)
	return
}

func (self *Homebrew) Qu(keywords []string) (err error) {
	out, err := exec.Command("brew", append([]string{"outdated"}, keywords...)...).Output()
	if err != nil {
		return
	}
	fmt.Printf("%s\n", out)
	return
}

func (self *Homebrew) R(keywords []string) (err error) {
	// TODO: better remove (deal with `homebrew cask uninstall`)
	out, err := exec.Command("brew", append([]string{"remove"}, keywords...)...).Output()
	if err != nil {
		return
	}
	fmt.Printf("%s\n", out)
	return
}

func (self *Homebrew) Rn(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Rns(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Rs(keywords []string) (err error) {
	// TODO: implement -Rs
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) S(keywords []string) (err error) {
	// TODO: better install (deal with `homebrew cask install`)
	out, err := exec.Command("brew", append([]string{"install"}, keywords...)...).Output()
	if err != nil {
		return
	}
	fmt.Printf("%s\n", out)
	return
}

func (self *Homebrew) Sc(keywords []string) (err error) {
	out, err := exec.Command("brew", append([]string{"cleanup"}, keywords...)...).Output()
	if err != nil {
		return
	}
	fmt.Printf("%s\n", out)
	return
}

func (self *Homebrew) Scc(keywords []string) (err error) {
	out, err := exec.Command("brew", append([]string{"cleanup", "-s"}, keywords...)...).Output()
	if err != nil {
		return
	}
	fmt.Printf("%s\n", out)
	return
}

func (self *Homebrew) Sccc(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Sg(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Si(keywords []string) (err error) {
	out, err := exec.Command("brew", append([]string{"info"}, keywords...)...).Output()
	if err != nil {
		return
	}
	fmt.Printf("%s\n", out)
	return
}

func (self *Homebrew) Sii(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Sl(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Ss(keywords []string) (err error) {
	out, err := exec.Command("brew", append([]string{"search"}, keywords...)...).Output()
	if err != nil {
		return
	}
	fmt.Printf("%s\n", out)
	return
}

func (self *Homebrew) Su(keywords []string) (err error) {
	out, err := exec.Command("brew", append([]string{"upgrade"}, keywords...)...).Output()
	if err != nil {
		return
	}
	fmt.Printf("%s\n", out)
	return
}

func (self *Homebrew) Suy(keywords []string) (err error) {
	err = self.Sy(keywords)
	if err != nil {
		return
	}
	err = self.Su(keywords)
	return
}

func (self *Homebrew) Sw(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Sy(keywords []string) (err error) {
	out, err := exec.Command("brew", append([]string{"update"}, keywords...)...).Output()
	if err != nil {
		return
	}
	fmt.Printf("%s\n", out)
	return
}

func (self *Homebrew) U(keywords []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}
