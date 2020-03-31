package dispatch

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
	return RunCommand(cmd)
}

// Q generates a list of installed packages.
func (pm *Dpkg) Q(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"dpkg", "-l"}, kws...))
}

// Qc shows the changelog of a package.
func (pm *Dpkg) Qc(kws []string) (err error) {
	return NotImplemented()
}

// Qe lists packages installed explicitly (not as dependencies).
func (pm *Dpkg) Qe(kws []string) (err error) {
	return NotImplemented()
}

// Qi displays local package information: name, version, description, etc.
func (pm *Dpkg) Qi(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"dpkg-query", "-s"}, kws...))
}

// Qk verifies one or more packages.
func (pm *Dpkg) Qk(kws []string) (err error) {
	return NotImplemented()
}

// Ql displays files provided by local package.
func (pm *Dpkg) Ql(kws []string) (err error) {
	return NotImplemented()
}

// Qm lists packages that are installed but are not available in any installation source (anymore).
func (pm *Dpkg) Qm(kws []string) (err error) {
	return NotImplemented()
}

// Qo queries the package which provides FILE.
func (pm *Dpkg) Qo(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"dpkg-query", "-S"}, kws...))
}

// Qp queries a package supplied on the command line rather than an entry in the package management database.
func (pm *Dpkg) Qp(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"dpkg-deb", "-I"}, kws...))
}

// Qs searches locally installed package for names or descriptions.
func (pm *Dpkg) Qs(kws []string) (err error) {
	return NotImplemented()
}

// Qu lists packages which have an update available.
func (pm *Dpkg) Qu(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "upgrade", "--trivial-only"}, kws...))
}

// R removes a single package, leaving all of its dependencies installed.
func (pm *Dpkg) R(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "remove"}, kws...))
}

// Rn removes a package and skips the generation of configuration backup files.
func (pm *Dpkg) Rn(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "purge"}, kws...))
}

// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
func (pm *Dpkg) Rns(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "autoremove", "--purge"}, kws...))
}

// Rs removes a package and its dependencies which are not required by any other installed package.
func (pm *Dpkg) Rs(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "autoremove"}, kws...))
}

// S installs one or more packages by name.
func (pm *Dpkg) S(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "install"}, kws...))
}

// Sc removes all the cached packages that are not currently installed, and the unused sync database.
func (pm *Dpkg) Sc(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "clean"}, kws...))
}

// Scc removes all files from the cache.
func (pm *Dpkg) Scc(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "autoclean"}, kws...))
}

// Sccc ...
// ! What is this?
func (pm *Dpkg) Sccc(kws []string) (err error) {
	return NotImplemented()
}

// Sg lists all packages belonging to the GROUP.
func (pm *Dpkg) Sg(kws []string) (err error) {
	return NotImplemented()
}

// Si displays remote package information: name, version, description, etc.
func (pm *Dpkg) Si(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-cache", "show"}, kws...))
}

// Sii displays packages which require X to be installed, aka reverse dependencies.
func (pm *Dpkg) Sii(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-cache", "rdepends"}, kws...))
}

// Sl displays a list of all packages in all installation sources that are handled by the packages management.
func (pm *Dpkg) Sl(kws []string) (err error) {
	return NotImplemented()
}

// Ss searches for package(s) by searching the expression in name, description, short description.
func (pm *Dpkg) Ss(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "search"}, kws...))
}

// Su updates outdated packages.
func (pm *Dpkg) Su(kws []string) (err error) {
	if err = pm.RunIfNotDry(append([]string{"apt-get", "upgrade"}, kws...)); err != nil {
		return
	}
	return pm.RunIfNotDry(append([]string{"apt-get", "dist-upgrade"}, kws...))
}

// Suy refreshes the local package database, then updates outdated packages.
func (pm *Dpkg) Suy(kws []string) (err error) {
	if err = pm.Sy(kws); err != nil {
		return
	}
	return pm.Su(kws)
}

// Sw retrieves all packages from the server, but does not install/upgrade anything.
func (pm *Dpkg) Sw(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "install", "--download-only"}, kws...))
}

// Sy refreshes the local package database.
func (pm *Dpkg) Sy(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"apt-get", "update"}, kws...))
}

// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
func (pm *Dpkg) U(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"dpkg", "-i"}, kws...))
}
