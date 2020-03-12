package dispatch

import (
	"fmt"
	"os/exec"
	"strings"
)

// Homebrew package manager config
type Homebrew struct{}

// For method implementation see: https://golang.org/src/os/exec/example_test.go

// TODO: for now the output will only be printed out when an operation is finished, which is disturbing. Fix that.

func (self *Homebrew) Q(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Qc(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "log"}, kw...))
}

func (self *Homebrew) Qe(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Qi(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "info"}, kw...))
}

func (self *Homebrew) Qk(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Ql(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "list"}, kw...))
}

func (self *Homebrew) Qm(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Qo(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Qp(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Qs(kw []string) (err error) {
	// TODO: it seems that the output of `brew list python` in fish has a mechanism against duplication:
	// /usr/local/Cellar/python/3.6.0/Frameworks/Python.framework/ (1234 files)
	return RunCommand(append([]string{"brew", "list"}, kw...))
}

func (self *Homebrew) Qu(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "outdated"}, kw...))
}

func (self *Homebrew) R(kw []string) (err error) {
	// TODO: better remove (deal with `homebrew cask uninstall`)
	out, err := exec.Command("brew", append([]string{"remove"}, kw...)...).CombinedOutput()
	fmt.Printf("%s\n", out)
	if err != nil {
		return
	}
	return
}

func (self *Homebrew) Rn(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Rns(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Rs(kw []string) (err error) {
	// TODO: implement -Rs
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) S(kw []string) (err error) {
	// TODO: better install (deal with `homebrew cask install`)
	install := func(pack string) (err error) {
		out, err := exec.Command("brew", "install", pack).CombinedOutput()
		fmt.Printf("%s\n", out)
		if index := strings.Index(string(out), "brew cask install"); index != -1 {
			fmt.Printf(":: Now trying with brew/cask...\n")
			out, err = exec.Command("brew", "cask", "install", pack).CombinedOutput()
			fmt.Printf("%s\n", out)
		}
		return
	}

	for _, pack := range kw {
		err = install(pack)
		if err != nil {
			return
		}
	}

	return
}

func (self *Homebrew) Sc(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "cleanup"}, kw...))
}

func (self *Homebrew) Scc(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "cleanup", "-s"}, kw...))
}

func (self *Homebrew) Sccc(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Sg(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Si(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "info"}, kw...))
}

func (self *Homebrew) Sii(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Sl(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Ss(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "search"}, kw...))
}

func (self *Homebrew) Su(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "upgrade"}, kw...))
}

func (self *Homebrew) Suy(kw []string) (err error) {
	err = self.Sy(kw)
	if err != nil {
		return
	}
	err = self.Su(kw)
	return
}

func (self *Homebrew) Sw(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

func (self *Homebrew) Sy(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "update"}, kw...))
}

func (self *Homebrew) U(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}
