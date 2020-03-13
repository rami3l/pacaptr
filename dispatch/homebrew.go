package dispatch

import (
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"
)

// Homebrew package manager config.
type Homebrew struct{}

// For method implementation see: https://golang.org/src/os/exec/example_test.go
// For method explanation see: https://wiki.archlinux.org/index.php/Pacman/Rosetta and https://wiki.archlinux.org/index.php/Pacman

// TODO: for now the output will only be printed out when an operation is finished, which is disturbing. Fix that.

// Q generates a list of installed packages.
func (hb *Homebrew) Q(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

// Qc shows the changelog of a package
func (hb *Homebrew) Qc(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "log"}, kw...))
}

// Qe lists packages installed explicitly (not as dependencies).
func (hb *Homebrew) Qe(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

// Qi displays local package information: Name, version, description, etc.
func (hb *Homebrew) Qi(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "info"}, kw...))
}

// Qk verifies one or more packages.
func (hb *Homebrew) Qk(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

// Ql displays files provided by local package.
func (hb *Homebrew) Ql(kw []string) (err error) {
	// TODO: it seems that the output of `brew list python` in fish has a mechanism against duplication:
	// /usr/local/Cellar/python/3.6.0/Frameworks/Python.framework/ (1234 files)
	return RunCommand(append([]string{"brew", "list"}, kw...))
}

// Qm lists packages that are installed but are not available in any installation source (anymore).
func (hb *Homebrew) Qm(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

// Qo queries the package which provides FILE.
func (hb *Homebrew) Qo(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

// Qp queries a package supplied on the command line rather than an entry in the package management database.
func (hb *Homebrew) Qp(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

// Qs searches locally installed package for names or descriptions.
func (hb *Homebrew) Qs(kw []string) (err error) {
	// TODO: it seems that the output of `brew list python` in fish has a mechanism against duplication:
	// /usr/local/Cellar/python/3.6.0/Frameworks/Python.framework/ (1234 files)
	return RunCommand(append([]string{"brew", "list"}, kw...))
}

// Qu lists packages which have an update available.
func (hb *Homebrew) Qu(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "outdated"}, kw...))
}

// R removes a single package, leaving all of its dependencies installed.
func (hb *Homebrew) R(kw []string) (err error) {
	uninstall := func(pack string) (err error) {
		var outBuf strings.Builder
		p := exec.Command("brew", "uninstall", pack)
		p.Stdout = io.MultiWriter(os.Stdout, &outBuf)
		p.Stderr = io.MultiWriter(os.Stderr, &outBuf)
		err = p.Run()
		if index := strings.Index(outBuf.String(), "Error: No such keg:"); index != -1 {
			fmt.Printf(":: `%s` is not installed or installed with brew/cask.\n", pack)
			fmt.Printf(":: Now trying with brew/cask...\n")
			alt := exec.Command("brew", "cask", "uninstall", pack)
			alt.Stdout = os.Stdout
			alt.Stderr = os.Stderr
			err = alt.Run()
		}
		return
	}

	for _, pack := range kw {
		err = uninstall(pack)
		if err != nil {
			return
		}
	}

	return
}

// Rn removes a package and skips the generation of configuration backup files.
func (hb *Homebrew) Rn(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
func (hb *Homebrew) Rns(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

// Rs removes a package and its dependencies which are not required by any other installed package.
func (hb *Homebrew) Rs(kw []string) (err error) {
	// TODO: implement -Rs
	return fmt.Errorf("pacapt: Feature not implemented")
}

// S installs one or more packages by name.
func (hb *Homebrew) S(kw []string) (err error) {
	install := func(pack string) (err error) {
		var outBuf strings.Builder
		p := exec.Command("brew", "install", pack)
		p.Stdout = io.MultiWriter(os.Stdout, &outBuf)
		p.Stderr = io.MultiWriter(os.Stderr, &outBuf)
		err = p.Run()
		if index := strings.Index(outBuf.String(), "brew cask install"); index != -1 {
			fmt.Printf(":: Now trying with brew/cask...\n")
			alt := exec.Command("brew", "cask", "install", pack)
			alt.Stdout = os.Stdout
			alt.Stderr = os.Stderr
			err = alt.Run()
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

// Sc removes all the cached packages that are not currently installed, and the unused sync database.
func (hb *Homebrew) Sc(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "cleanup"}, kw...))
}

// Scc removes all files from the cache.
func (hb *Homebrew) Scc(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "cleanup", "-s"}, kw...))
}

// Sccc ...
// ! What is this?
func (hb *Homebrew) Sccc(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

// Sg lists all packages belonging to the GROUP.
func (hb *Homebrew) Sg(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

// Si displays remote package information: Name, version, description, etc.
func (hb *Homebrew) Si(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "info"}, kw...))
}

// Sii displays packages which require X to be installed, aka reverse dependencies.
func (hb *Homebrew) Sii(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

// Sl displays a list of all packages in all installation sources that are handled by the packages management.
func (hb *Homebrew) Sl(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

// Ss searches for package(s) by searching the expression in name, description, short description.
func (hb *Homebrew) Ss(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "search"}, kw...))
}

// Su updates outdated packages.
func (hb *Homebrew) Su(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "upgrade"}, kw...))
}

// Suy refreshes the local package database, then updates outdated packages.
func (hb *Homebrew) Suy(kw []string) (err error) {
	err = hb.Sy(kw)
	if err != nil {
		return
	}
	err = hb.Su(kw)
	return
}

// Sw retrieves all packages from the server, but does not install/upgrade anything.
func (hb *Homebrew) Sw(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}

// Sy refreshes the local package database.
func (hb *Homebrew) Sy(kw []string) (err error) {
	return RunCommand(append([]string{"brew", "update"}, kw...))
}

// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
func (hb *Homebrew) U(kw []string) (err error) {
	return fmt.Errorf("pacapt: Feature not implemented")
}
