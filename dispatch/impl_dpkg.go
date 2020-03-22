package dispatch

import (
	"io"
	"os"
	"os/exec"
	"strings"
)

// Dpkg package manager config.
type Dpkg struct {
	DryRun    bool
	NoConfirm bool
}

// For method implementation see: https://golang.org/src/os/exec/example_test.go
// For method explanation see: https://wiki.archlinux.org/index.php/Pacman/Rosetta
// and https://wiki.archlinux.org/index.php/Pacman

// RunIfNotDry prints out the command if DryRun, else it runs the command.
func (pm *Dpkg) RunIfNotDry(cmd []string) (err error) {
	if pm.DryRun {
		PrintCommand(cmd)
		return
	}
	RunCommand(cmd)
	return
}

// CheckOutput runs the command and returns its output both to a string and to Stdout (ignored if DryRun).
func (pm *Dpkg) CheckOutput(cmd []string) (out string, err error) {
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
func (pm *Dpkg) Q(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"dpkg", "-l"}, kw...))
}

// Qc shows the changelog of a package.
func (pm *Dpkg) Qc(kw []string) (err error) {
	return NotImplemented()
}

// Qe lists packages installed explicitly (not as dependencies).
func (pm *Dpkg) Qe(kw []string) (err error) {
	return NotImplemented()
}

// Qi displays local package information: name, version, description, etc.
func (pm *Dpkg) Qi(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"dpkg-query", "-s"}, kw...))
}

// Qk verifies one or more packages.
func (pm *Dpkg) Qk(kw []string) (err error) {
	return NotImplemented()
}

// Ql displays files provided by local package.
func (pm *Dpkg) Ql(kw []string) (err error) {
	return NotImplemented()
}

// Qm lists packages that are installed but are not available in any installation source (anymore).
func (pm *Dpkg) Qm(kw []string) (err error) {
	return NotImplemented()
}

// Qo queries the package which provides FILE.
func (pm *Dpkg) Qo(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"dpkg-query", "-S"}, kw...))
}

// Qp queries a package supplied on the command line rather than an entry in the package management database.
func (pm *Dpkg) Qp(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"dpkg-deb", "-I"}, kw...))
}

// Qs searches locally installed package for names or descriptions.
func (pm *Dpkg) Qs(kw []string) (err error) {
	return NotImplemented()
}

// Qu lists packages which have an update available.
func (pm *Dpkg) Qu(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "upgrade", "--trivial-only"}, kw...))
}

// R removes a single package, leaving all of its dependencies installed.
func (pm *Dpkg) R(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "remove"}, kw...))
}

// Rn removes a package and skips the generation of configuration backup files.
func (pm *Dpkg) Rn(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "purge"}, kw...))
}

// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
func (pm *Dpkg) Rns(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "--purge", "autoremove"}, kw...))
}

// Rs removes a package and its dependencies which are not required by any other installed package.
func (pm *Dpkg) Rs(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "autoremove"}, kw...))
}

// S installs one or more packages by name.
func (pm *Dpkg) S(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "install"}, kw...))
}

// Sc removes all the cached packages that are not currently installed, and the unused sync database.
func (pm *Dpkg) Sc(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "clean"}, kw...))
}

// Scc removes all files from the cache.
func (pm *Dpkg) Scc(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "autoclean"}, kw...))
}

// Sccc ...
// ! What is this?
func (pm *Dpkg) Sccc(kw []string) (err error) {
	return NotImplemented()
}

// Sg lists all packages belonging to the GROUP.
func (pm *Dpkg) Sg(kw []string) (err error) {
	return NotImplemented()
}

// Si displays remote package information: name, version, description, etc.
func (pm *Dpkg) Si(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-cache", "show"}, kw...))
}

// Sii displays packages which require X to be installed, aka reverse dependencies.
func (pm *Dpkg) Sii(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-cache", "rdepends"}, kw...))
}

// Sl displays a list of all packages in all installation sources that are handled by the packages management.
func (pm *Dpkg) Sl(kw []string) (err error) {
	return NotImplemented()
}

// Ss searches for package(s) by searching the expression in name, description, short description.
func (pm *Dpkg) Ss(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "search"}, kw...))
}

// Su updates outdated packages.
func (pm *Dpkg) Su(kw []string) (err error) {
	if err = pm.RunIfNotDry(append([]string{"apt-get", "upgrade"}, kw...)); err != nil {
		return
	}
	err = pm.RunIfNotDry(append([]string{"apt-get", "dist-upgrade"}, kw...))
	return
}

// Suy refreshes the local package database, then updates outdated packages.
func (pm *Dpkg) Suy(kw []string) (err error) {
	if err = pm.Sy(kw); err != nil {
		return
	}
	err = pm.Su(kw)
	return
}

// Sw retrieves all packages from the server, but does not install/upgrade anything.
func (pm *Dpkg) Sw(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "--download-only", "install"}, kw...))
}

// Sy refreshes the local package database.
func (pm *Dpkg) Sy(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "update"}, kw...))
}

// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
func (pm *Dpkg) U(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"dpkg", "-i"}, kw...))
}
