package dispatch

import (
	"io"
	"os"
	"os/exec"
	"strings"
)

// Unknown package manager config.
// This is a special "blank" PackManager reserved for copy-paste convenience
// as well as unknown package manager handling.
type Unknown struct {
	DryRun    bool
	NoConfirm bool
}

// For method implementation see: https://golang.org/src/os/exec/example_test.go
// For method explanation see: https://wiki.archlinux.org/index.php/Pacman/Rosetta
// and https://wiki.archlinux.org/index.php/Pacman

// RunIfNotDry prints out the command if DryRun, else it runs the command.
func (unk *Unknown) RunIfNotDry(cmd []string) (err error) {
	if unk.DryRun {
		PrintCommand(cmd)
		return
	}
	RunCommand(cmd)
	return
}

// CheckOutput runs the command and returns its output both to a string and to Stdout(ignored if DryRun).
func (unk *Unknown) CheckOutput(cmd []string) (out string, err error) {
	var outBuf strings.Builder
	PrintCommand(cmd)
	p := exec.Command(cmd[0], cmd[1:]...)
	if unk.DryRun {
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
func (unk *Unknown) Q(kw []string) (err error) {
	return NotImplemented()
}

// Qc shows the changelog of a package.
func (unk *Unknown) Qc(kw []string) (err error) {
	return NotImplemented()
}

// Qe lists packages installed explicitly (not as dependencies).
func (unk *Unknown) Qe(kw []string) (err error) {
	return NotImplemented()
}

// Qi displays local package information: name, version, description, etc.
func (unk *Unknown) Qi(kw []string) (err error) {
	return NotImplemented()
}

// Qk verifies one or more packages.
func (unk *Unknown) Qk(kw []string) (err error) {
	return NotImplemented()
}

// Ql displays files provided by local package.
func (unk *Unknown) Ql(kw []string) (err error) {
	return NotImplemented()
}

// Qm lists packages that are installed but are not available in any installation source (anymore).
func (unk *Unknown) Qm(kw []string) (err error) {
	return NotImplemented()
}

// Qo queries the package which provides FILE.
func (unk *Unknown) Qo(kw []string) (err error) {
	return NotImplemented()
}

// Qp queries a package supplied on the command line rather than an entry in the package management database.
func (unk *Unknown) Qp(kw []string) (err error) {
	return NotImplemented()
}

// Qs searches locally installed package for names or descriptions.
func (unk *Unknown) Qs(kw []string) (err error) {
	return NotImplemented()
}

// Qu lists packages which have an update available.
func (unk *Unknown) Qu(kw []string) (err error) {
	return NotImplemented()
}

// R removes a single package, leaving all of its dependencies installed.
func (unk *Unknown) R(kw []string) (err error) {
	return NotImplemented()
}

// Rn removes a package and skips the generation of configuration backup files.
func (unk *Unknown) Rn(kw []string) (err error) {
	return NotImplemented()
}

// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
func (unk *Unknown) Rns(kw []string) (err error) {
	return NotImplemented()
}

// Rs removes a package and its dependencies which are not required by any other installed package.
func (unk *Unknown) Rs(kw []string) (err error) {
	return NotImplemented()
}

// S installs one or more packages by name.
func (unk *Unknown) S(kw []string) (err error) {
	return NotImplemented()
}

// Sc removes all the cached packages that are not currently installed, and the unused sync database.
func (unk *Unknown) Sc(kw []string) (err error) {
	return NotImplemented()
}

// Scc removes all files from the cache.
func (unk *Unknown) Scc(kw []string) (err error) {
	return NotImplemented()
}

// Sccc ...
// ! What is this?
func (unk *Unknown) Sccc(kw []string) (err error) {
	return NotImplemented()
}

// Sg lists all packages belonging to the GROUP.
func (unk *Unknown) Sg(kw []string) (err error) {
	return NotImplemented()
}

// Si displays remote package information: name, version, description, etc.
func (unk *Unknown) Si(kw []string) (err error) {
	return NotImplemented()
}

// Sii displays packages which require X to be installed, aka reverse dependencies.
func (unk *Unknown) Sii(kw []string) (err error) {
	return NotImplemented()
}

// Sl displays a list of all packages in all installation sources that are handled by the packages management.
func (unk *Unknown) Sl(kw []string) (err error) {
	return NotImplemented()
}

// Ss searches for package(s) by searching the expression in name, description, short description.
func (unk *Unknown) Ss(kw []string) (err error) {
	return NotImplemented()
}

// Su updates outdated packages.
func (unk *Unknown) Su(kw []string) (err error) {
	return NotImplemented()
}

// Suy refreshes the local package database, then updates outdated packages.
func (unk *Unknown) Suy(kw []string) (err error) {
	return NotImplemented()
}

// Sw retrieves all packages from the server, but does not install/upgrade anything.
func (unk *Unknown) Sw(kw []string) (err error) {
	return NotImplemented()
}

// Sy refreshes the local package database.
func (unk *Unknown) Sy(kw []string) (err error) {
	return NotImplemented()
}

// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
func (unk *Unknown) U(kw []string) (err error) {
	return NotImplemented()
}
