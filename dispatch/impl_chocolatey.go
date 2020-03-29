package dispatch

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
func (pm *Chocolatey) RunIfNotDry(cmd []string) (err error) {
	if pm.DryRun {
		PrintCommand(cmd)
		return
	}
	return RunCommand(cmd)
}

// Q generates a list of installed packages.
func (pm *Chocolatey) Q(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"choco", "list", "--localonly"}, kw...))
}

// Qc shows the changelog of a package.
func (pm *Chocolatey) Qc(kw []string) (err error) {
	return NotImplemented()
}

// Qe lists packages installed explicitly (not as dependencies).
func (pm *Chocolatey) Qe(kw []string) (err error) {
	return NotImplemented()
}

// Qi displays local package information: name, version, description, etc.
func (pm *Chocolatey) Qi(kw []string) (err error) {
	return pm.Si(kw)
}

// Qk verifies one or more packages.
func (pm *Chocolatey) Qk(kw []string) (err error) {
	return NotImplemented()
}

// Ql displays files provided by local package.
func (pm *Chocolatey) Ql(kw []string) (err error) {
	return NotImplemented()
}

// Qm lists packages that are installed but are not available in any installation source (anymore).
func (pm *Chocolatey) Qm(kw []string) (err error) {
	return NotImplemented()
}

// Qo queries the package which provides FILE.
func (pm *Chocolatey) Qo(kw []string) (err error) {
	return NotImplemented()
}

// Qp queries a package supplied on the command line rather than an entry in the package management database.
func (pm *Chocolatey) Qp(kw []string) (err error) {
	return NotImplemented()
}

// Qs searches locally installed package for names or descriptions.
func (pm *Chocolatey) Qs(kw []string) (err error) {
	return NotImplemented()
}

// Qu lists packages which have an update available.
func (pm *Chocolatey) Qu(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"choco", "outdated"}, kw...))
}

// R removes a single package, leaving all of its dependencies installed.
func (pm *Chocolatey) R(kw []string) (err error) {
	var cmd []string
	if pm.NoConfirm {
		cmd = append([]string{"choco", "uninstall", "--yes"}, kw...)
	} else {
		cmd = append([]string{"choco", "uninstall"}, kw...)
	}
	return pm.RunIfNotDry(cmd)
}

// Rn removes a package and skips the generation of configuration backup files.
func (pm *Chocolatey) Rn(kw []string) (err error) {
	return NotImplemented()
}

// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
func (pm *Chocolatey) Rns(kw []string) (err error) {
	return NotImplemented()
}

// Rs removes a package and its dependencies which are not required by any other installed package.
func (pm *Chocolatey) Rs(kw []string) (err error) {
	var cmd []string
	if pm.NoConfirm {
		cmd = append([]string{"choco", "uninstall", "--removedependencies", "--yes"}, kw...)
	} else {
		cmd = append([]string{"choco", "uninstall", "--removedependencies"}, kw...)
	}
	return pm.RunIfNotDry(cmd)
}

// S installs one or more packages by name.
func (pm *Chocolatey) S(kw []string) (err error) {
	var cmd []string
	if pm.NoConfirm {
		cmd = append([]string{"choco", "install", "--yes"}, kw...)
	} else {
		cmd = append([]string{"choco", "install"}, kw...)
	}
	return pm.RunIfNotDry(cmd)
}

// Sc removes all the cached packages that are not currently installed, and the unused sync database.
func (pm *Chocolatey) Sc(kw []string) (err error) {
	return NotImplemented()
}

// Scc removes all files from the cache.
func (pm *Chocolatey) Scc(kw []string) (err error) {
	return NotImplemented()
}

// Sccc ...
// ! What is this?
func (pm *Chocolatey) Sccc(kw []string) (err error) {
	return NotImplemented()
}

// Sg lists all packages belonging to the GROUP.
func (pm *Chocolatey) Sg(kw []string) (err error) {
	return NotImplemented()
}

// Si displays remote package information: name, version, description, etc.
func (pm *Chocolatey) Si(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"choco", "info"}, kw...))
}

// Sii displays packages which require X to be installed, aka reverse dependencies.
func (pm *Chocolatey) Sii(kw []string) (err error) {
	return NotImplemented()
}

// Sl displays a list of all packages in all installation sources that are handled by the packages management.
func (pm *Chocolatey) Sl(kw []string) (err error) {
	return NotImplemented()
}

// Ss searches for package(s) by searching the expression in name, description, short description.
func (pm *Chocolatey) Ss(kw []string) (err error) {
	return pm.RunIfNotDry(append([]string{"choco", "search"}, kw...))
}

// Su updates outdated packages.
func (pm *Chocolatey) Su(kw []string) (err error) {
	if len(kw) == 0 {
		kw = []string{"all"}
	}
	var cmd []string
	if pm.NoConfirm {
		cmd = append([]string{"choco", "upgrade", "--yes"}, kw...)
	} else {
		cmd = append([]string{"choco", "upgrade"}, kw...)
	}
	return pm.RunIfNotDry(cmd)
}

// Suy refreshes the local package database, then updates outdated packages.
func (pm *Chocolatey) Suy(kw []string) (err error) {
	return pm.Su(kw)
}

// Sw retrieves all packages from the server, but does not install/upgrade anything.
func (pm *Chocolatey) Sw(kw []string) (err error) {
	return NotImplemented()
}

// Sy refreshes the local package database.
func (pm *Chocolatey) Sy(kw []string) (err error) {
	return NotImplemented()
}

// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
func (pm *Chocolatey) U(kw []string) (err error) {
	return NotImplemented()
}
