package dispatch

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"
)

// Homebrew package manager config.
type Homebrew struct {
	DryRun bool
}

// For method implementation see: https://golang.org/src/os/exec/example_test.go
// For method explanation see: https://wiki.archlinux.org/index.php/Pacman/Rosetta
// and https://wiki.archlinux.org/index.php/Pacman

// RunIfNotDry prints out the command if DryRun, else it runs the command.
func (pm *Homebrew) RunIfNotDry(cmd []string) (err error) {
	if pm.DryRun {
		PrintCommand(cmd)
		return
	}
	RunCommand(cmd)
	return
}

// CheckOutput runs the command and returns its output both to a string and to Stdout (ignored if DryRun).
func (pm *Homebrew) CheckOutput(cmd []string) (out string, err error) {
	var outBuf strings.Builder
	PrintCommand(cmd)
	p := exec.Command(cmd[0], cmd[1:]...)
	p.Stdin = os.Stdin
	if pm.DryRun {
		p.Stdout = &outBuf
		p.Stderr = &outBuf
	} else {
		p.Stdout = io.MultiWriter(os.Stdout, &outBuf)
		p.Stderr = io.MultiWriter(os.Stderr, &outBuf)
	}
	err = p.Run()
	out = outBuf.String()
	return
}

// Q generates a list of installed packages.
func (pm *Homebrew) Q(kw []string) (err error) {
	return NotImplemented()
}

// Qc shows the changelog of a package.
func (pm *Homebrew) Qc(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"brew", "log"}, kw...))
}

// Qe lists packages installed explicitly (not as dependencies).
func (pm *Homebrew) Qe(kw []string) (err error) {
	return NotImplemented()
}

// Qi displays local package information: name, version, description, etc.
func (pm *Homebrew) Qi(kw []string) (err error) {
	return pm.Si(kw)
}

// Qk verifies one or more packages.
func (pm *Homebrew) Qk(kw []string) (err error) {
	return NotImplemented()
}

// Ql displays files provided by local package.
func (pm *Homebrew) Ql(kw []string) (err error) {
	// TODO: it seems that the output of `brew list python` in fish has a mechanism against duplication:
	// /usr/local/Cellar/python/3.6.0/Frameworks/Python.framework/ (1234 files)
	return pm.RunIfNotDry(append([]string{"brew", "list"}, kw...))
}

// Qm lists packages that are installed but are not available in any installation source (anymore).
func (pm *Homebrew) Qm(kw []string) (err error) {
	return NotImplemented()
}

// Qo queries the package which provides FILE.
func (pm *Homebrew) Qo(kw []string) (err error) {
	return NotImplemented()
}

// Qp queries a package supplied on the command line rather than an entry in the package management database.
func (pm *Homebrew) Qp(kw []string) (err error) {
	return NotImplemented()
}

// Qs searches locally installed package for names or descriptions.
func (pm *Homebrew) Qs(kw []string) (err error) {
	outBytes, err := exec.Command("brew", "list").Output()
	out := fmt.Sprintf("%s", outBytes)
	scanner := bufio.NewScanner(strings.NewReader(out))
	for scanner.Scan() {
		line := scanner.Text()
		if i := strings.Index(line, strings.Join(kw, " ")); i != -1 {
			fmt.Printf("%s\n", line)
		}
	}
	return
}

// Qu lists packages which have an update available.
func (pm *Homebrew) Qu(kw []string) (err error) {
	outBytes, err := exec.Command("brew", "outdated").Output()
	out := fmt.Sprintf("%s", outBytes)
	scanner := bufio.NewScanner(strings.NewReader(out))
	for scanner.Scan() {
		line := scanner.Text()
		if i := strings.Index(line, strings.Join(kw, " ")); i != -1 {
			fmt.Printf("%s\n", line)
		}
	}
	return
}

// R removes a single package, leaving all of its dependencies installed.
func (pm *Homebrew) R(kw []string) (err error) {
	uninstall := func(pack string) (err error) {
		out, err := pm.CheckOutput([]string{"brew", "uninstall", pack})

		// fallback when `brew uninstall` fails
		if index := strings.Index(out, "Error: No such keg:"); index != -1 {
			fmt.Printf(":: `%s` is not installed or installed with brew/cask.\n", pack)
			fmt.Printf(":: Now trying with brew/cask...\n")
			err = pm.RunIfNotDry([]string{"brew", "cask", "uninstall", pack})
		}

		return
	}

	for _, pack := range kw {
		if err = uninstall(pack); err != nil {
			return
		}
	}

	return
}

// Rn removes a package and skips the generation of configuration backup files.
func (pm *Homebrew) Rn(kw []string) (err error) {
	return NotImplemented()
}

// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
func (pm *Homebrew) Rns(kw []string) (err error) {
	return NotImplemented()
}

// Rs removes a package and its dependencies which are not required by any other installed package.
func (pm *Homebrew) Rs(kw []string) (err error) {
	// TODO: implement -Rs
	return NotImplemented()
}

// S installs one or more packages by name.
func (pm *Homebrew) S(kw []string) (err error) {
	const (
		notFound      = iota
		caskNotNeeded = iota
		caskNeeded    = iota
	)

	search := func(pack string) (code int, err error) {
		p := exec.Command("brew", "info", pack)
		outbytes, err := p.CombinedOutput()
		out := fmt.Sprintf("%s", outbytes)
		// fmt.Print(out)

		code = caskNotNeeded
		if i := strings.Index(out, "Error: No available formula with the name"); i != -1 {
			code = notFound
			if j := strings.Index(out, "Found a cask named"); j != -1 {
				code = caskNeeded
			}
			err = nil
		}

		// fmt.Printf("Code: %v\n", code)
		return
	}

	install := func(pack string) (err error) {
		code, err := search(pack)
		if err != nil {
			return
		}

		switch code {
		case notFound, caskNotNeeded:
			return pm.RunIfNotDry([]string{"brew", "install", pack})
		case caskNeeded:
			return pm.RunIfNotDry([]string{"brew", "cask", "install", pack})
		}

		return
	}

	for _, pack := range kw {
		if err = install(pack); err != nil {
			return
		}
	}

	return
}

// Sc removes all the cached packages that are not currently installed, and the unused sync database.
func (pm *Homebrew) Sc(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"brew", "cleanup"}, kw...))
}

// Scc removes all files from the cache.
func (pm *Homebrew) Scc(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"brew", "cleanup", "-s"}, kw...))
}

// Sccc ...
// ! What is this?
func (pm *Homebrew) Sccc(kw []string) (err error) {
	return NotImplemented()
}

// Sg lists all packages belonging to the GROUP.
func (pm *Homebrew) Sg(kw []string) (err error) {
	return NotImplemented()
}

// Si displays remote package information: name, version, description, etc.
func (pm *Homebrew) Si(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"brew", "info"}, kw...))
}

// Sii displays packages which require X to be installed, aka reverse dependencies.
func (pm *Homebrew) Sii(kw []string) (err error) {
	return NotImplemented()
}

// Sl displays a list of all packages in all installation sources that are handled by the packages management.
func (pm *Homebrew) Sl(kw []string) (err error) {
	return NotImplemented()
}

// Ss searches for package(s) by searching the expression in name, description, short description.
func (pm *Homebrew) Ss(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"brew", "search"}, kw...))
}

// Su updates outdated packages.
func (pm *Homebrew) Su(kw []string) (err error) {
	if err = pm.RunIfNotDry(append([]string{"brew", "upgrade"}, kw...)); err != nil {
		return
	}
	return pm.RunIfNotDry(append([]string{"brew", "cask", "upgrade"}, kw...))
}

// Suy refreshes the local package database, then updates outdated packages.
func (pm *Homebrew) Suy(kw []string) (err error) {
	if err = pm.Sy(kw); err != nil {
		return
	}
	return pm.Su(kw)
}

// Sw retrieves all packages from the server, but does not install/upgrade anything.
func (pm *Homebrew) Sw(kw []string) (err error) {
	return NotImplemented()
}

// Sy refreshes the local package database.
func (pm *Homebrew) Sy(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"brew", "update"}, kw...))
}

// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
func (pm *Homebrew) U(kw []string) (err error) {
	return NotImplemented()
}
