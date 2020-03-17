package dispatch

import (
	"io"
	"os"
	"os/exec"
	"strings"
)

// Chocolatey package manager config.
type Chocolatey struct {
	DryRun    bool
	NoConfirm bool
}

// Ported from: https://github.com/Grenadingue/batch-pacapt
// For method implementation see: https://golang.org/src/os/exec/example_test.go
// For method explanation see: https://wiki.archlinux.org/index.php/Pacman/Rosetta
// and https://wiki.archlinux.org/index.php/Pacman

// RunIfNotDry prints out the command if DryRun, else it runs the command.
func (ch *Chocolatey) RunIfNotDry(cmd []string) (err error) {
	if ch.DryRun {
		PrintCommand(cmd)
		return
	}
	RunCommand(cmd)
	return
}

// CheckOutput runs the command and returns its output both to a string and to Stdout (ignored if DryRun).
func (ch *Chocolatey) CheckOutput(cmd []string) (out string, err error) {
	var outBuf strings.Builder
	PrintCommand(cmd)
	p := exec.Command(cmd[0], cmd[1:]...)
	if ch.DryRun {
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
func (ch *Chocolatey) Q(kw []string) (err error) {
	return ch.RunIfNotDry(append([]string{"choco", "list", "--localonly"}, kw...))
}

// Qc shows the changelog of a package.
func (ch *Chocolatey) Qc(kw []string) (err error) {
	return NotImplemented()
}

// Qe lists packages installed explicitly (not as dependencies).
func (ch *Chocolatey) Qe(kw []string) (err error) {
	return NotImplemented()
}

// Qi displays local package information: name, version, description, etc.
func (ch *Chocolatey) Qi(kw []string) (err error) {
	return ch.Si(kw)
}

// Qk verifies one or more packages.
func (ch *Chocolatey) Qk(kw []string) (err error) {
	return NotImplemented()
}

// Ql displays files provided by local package.
func (ch *Chocolatey) Ql(kw []string) (err error) {
	return NotImplemented()
}

// Qm lists packages that are installed but are not available in any installation source (anymore).
func (ch *Chocolatey) Qm(kw []string) (err error) {
	return NotImplemented()
}

// Qo queries the package which provides FILE.
func (ch *Chocolatey) Qo(kw []string) (err error) {
	return NotImplemented()
}

// Qp queries a package supplied on the command line rather than an entry in the package management database.
func (ch *Chocolatey) Qp(kw []string) (err error) {
	return NotImplemented()
}

// Qs searches locally installed package for names or descriptions.
func (ch *Chocolatey) Qs(kw []string) (err error) {
	return NotImplemented()
}

// Qu lists packages which have an update available.
func (ch *Chocolatey) Qu(kw []string) (err error) {
	return ch.RunIfNotDry(append([]string{"choco", "outdated"}, kw...))
}

// R removes a single package, leaving all of its dependencies installed.
func (ch *Chocolatey) R(kw []string) (err error) {
	var cmd []string
	if ch.NoConfirm {
		cmd = append([]string{"choco", "uninstall", "--yes"}, kw...)
	} else {
		cmd = append([]string{"choco", "uninstall"}, kw...)
	}
	return ch.RunIfNotDry(cmd)
}

// Rn removes a package and skips the generation of configuration backup files.
func (ch *Chocolatey) Rn(kw []string) (err error) {
	return NotImplemented()
}

// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
func (ch *Chocolatey) Rns(kw []string) (err error) {
	return NotImplemented()
}

// Rs removes a package and its dependencies which are not required by any other installed package.
func (ch *Chocolatey) Rs(kw []string) (err error) {
	var cmd []string
	if ch.NoConfirm {
		cmd = append([]string{"choco", "uninstall", "--removedependencies", "--yes"}, kw...)
	} else {
		cmd = append([]string{"choco", "uninstall", "--removedependencies"}, kw...)
	}
	return ch.RunIfNotDry(cmd)
}

// S installs one or more packages by name.
func (ch *Chocolatey) S(kw []string) (err error) {
	var cmd []string
	if ch.NoConfirm {
		cmd = append([]string{"choco", "install", "--yes"}, kw...)
	} else {
		cmd = append([]string{"choco", "install"}, kw...)
	}
	return ch.RunIfNotDry(cmd)
}

// Sc removes all the cached packages that are not currently installed, and the unused sync database.
func (ch *Chocolatey) Sc(kw []string) (err error) {
	return NotImplemented()
}

// Scc removes all files from the cache.
func (ch *Chocolatey) Scc(kw []string) (err error) {
	return NotImplemented()
}

// Sccc ...
// ! What is this?
func (ch *Chocolatey) Sccc(kw []string) (err error) {
	return NotImplemented()
}

// Sg lists all packages belonging to the GROUP.
func (ch *Chocolatey) Sg(kw []string) (err error) {
	return NotImplemented()
}

// Si displays remote package information: name, version, description, etc.
func (ch *Chocolatey) Si(kw []string) (err error) {
	return ch.RunIfNotDry(append([]string{"choco", "info"}, kw...))
}

// Sii displays packages which require X to be installed, aka reverse dependencies.
func (ch *Chocolatey) Sii(kw []string) (err error) {
	return NotImplemented()
}

// Sl displays a list of all packages in all installation sources that are handled by the packages management.
func (ch *Chocolatey) Sl(kw []string) (err error) {
	return NotImplemented()
}

// Ss searches for package(s) by searching the expression in name, description, short description.
func (ch *Chocolatey) Ss(kw []string) (err error) {
	return ch.RunIfNotDry(append([]string{"choco", "search"}, kw...))
}

// Su updates outdated packages.
func (ch *Chocolatey) Su(kw []string) (err error) {
	var cmd []string
	if ch.NoConfirm {
		cmd = append([]string{"choco", "upgrade", "--yes"}, kw...)
	} else {
		cmd = append([]string{"choco", "upgrade"}, kw...)
	}
	return ch.RunIfNotDry(cmd)
}

// Suy refreshes the local package database, then updates outdated packages.
func (ch *Chocolatey) Suy(kw []string) (err error) {
	return ch.Su(kw)
}

// Sw retrieves all packages from the server, but does not install/upgrade anything.
func (ch *Chocolatey) Sw(kw []string) (err error) {
	return NotImplemented()
}

// Sy refreshes the local package database.
func (ch *Chocolatey) Sy(kw []string) (err error) {
	return NotImplemented()
}

// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
func (ch *Chocolatey) U(kw []string) (err error) {
	return ch.Su(kw)
}
