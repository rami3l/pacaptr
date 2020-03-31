package dispatch

import (
	"bufio"
	"fmt"
	"os/exec"
	"regexp"
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
	return RunCommand(cmd)
}

const (
	notFound = iota
	caskNotNeeded
	caskNeeded
)

// search determines if a package should be (un)installed with a `brew cask` command.
func (pm *Homebrew) search(pack string) (code int, err error) {
	p := exec.Command("brew", "info", pack)
	outbytes, err := p.CombinedOutput()
	out := fmt.Sprintf("%s", outbytes)
	// fmt.Print(out)

	code = caskNotNeeded
	if i := strings.Index(out, "No available formula with the name"); i != -1 {
		code = notFound
		if j := strings.Index(out, "Found a cask named"); j != -1 {
			code = caskNeeded
		}
		err = nil
	}

	// fmt.Printf("Code: %v\n", code)
	return
}

// Q generates a list of installed packages.
func (pm *Homebrew) Q(kws []string) (err error) {
	return NotImplemented()
}

// Qc shows the changelog of a package.
func (pm *Homebrew) Qc(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"brew", "log"}, kws...))
}

// Qe lists packages installed explicitly (not as dependencies).
func (pm *Homebrew) Qe(kws []string) (err error) {
	return NotImplemented()
}

// Qi displays local package information: name, version, description, etc.
func (pm *Homebrew) Qi(kws []string) (err error) {
	return pm.Si(kws)
}

// Qk verifies one or more packages.
func (pm *Homebrew) Qk(kws []string) (err error) {
	return NotImplemented()
}

// Ql displays files provided by local package.
func (pm *Homebrew) Ql(kws []string) (err error) {
	// TODO: it seems that the output of `brew list python` in fish has a mechanism against duplication:
	// /usr/local/Cellar/python/3.6.0/Frameworks/Python.framework/ (1234 files)
	return pm.RunIfNotDry(append([]string{"brew", "list"}, kws...))
}

// Qm lists packages that are installed but are not available in any installation source (anymore).
func (pm *Homebrew) Qm(kws []string) (err error) {
	return NotImplemented()
}

// Qo queries the package which provides FILE.
func (pm *Homebrew) Qo(kws []string) (err error) {
	// TODO: implement -Qo
	return NotImplemented()
}

// Qp queries a package supplied on the command line rather than an entry in the package management database.
func (pm *Homebrew) Qp(kws []string) (err error) {
	return NotImplemented()
}

// Qs searches locally installed package for names or descriptions.
func (pm *Homebrew) Qs(kws []string) (err error) {
	// According to https://www.archlinux.org/pacman/pacman.8.html#_query_options_apply_to_em_q_em_a_id_qo_a,
	// when including multiple search terms, only packages with descriptions matching ALL of those terms are returned.
	matchAll := func(str string, kws []string) (match bool) {
		for _, kw := range kws {
			if re := regexp.MustCompile(kw); !re.Match([]byte(str)) {
				return false
			}
		}
		return true
	}

	search := func(cmd []string) (err error) {
		PrintCommand(append(cmd))
		outBytes, err := exec.Command(cmd[0], cmd[1:]...).Output()
		out := fmt.Sprintf("%s", outBytes)
		scanner := bufio.NewScanner(strings.NewReader(out))
		for scanner.Scan() {
			line := scanner.Text()
			if matchAll(line, kws) {
				fmt.Printf("%s\n", line)
			}
		}
		return
	}

	if err = search([]string{"brew", "list"}); err != nil {
		return
	}
	return search([]string{"brew", "cask", "list"})
}

// Qu lists packages which have an update available.
func (pm *Homebrew) Qu(kws []string) (err error) {
	outBytes, err := exec.Command("brew", "outdated").Output()
	out := fmt.Sprintf("%s", outBytes)
	scanner := bufio.NewScanner(strings.NewReader(out))
	for scanner.Scan() {
		line := scanner.Text()
		if i := strings.Index(line, strings.Join(kws, " ")); i != -1 {
			fmt.Printf("%s\n", line)
		}
	}
	return
}

// R removes a single package, leaving all of its dependencies installed.
func (pm *Homebrew) R(kws []string) (err error) {
	uninstall := func(pack string) (err error) {
		code, err := pm.search(pack)
		if err != nil {
			return
		}

		switch code {
		case notFound, caskNotNeeded:
			return pm.RunIfNotDry([]string{"brew", "uninstall", pack})
		case caskNeeded:
			return pm.RunIfNotDry([]string{"brew", "cask", "uninstall", pack})
		}

		return
	}

	for _, pack := range kws {
		if err = uninstall(pack); err != nil {
			return
		}
	}

	return
}

// Rn removes a package and skips the generation of configuration backup files.
func (pm *Homebrew) Rn(kws []string) (err error) {
	return NotImplemented()
}

// Rns removes a package and its dependencies which are not required by any other installed package, and skips the generation of configuration backup files.
func (pm *Homebrew) Rns(kws []string) (err error) {
	return NotImplemented()
}

// Rs removes a package and its dependencies which are not required by any other installed package.
func (pm *Homebrew) Rs(kws []string) (err error) {
	// TODO: implement -Rs
	// ! Maybe we should just call `brew rmtree`
	if pm.DryRun {
		err = RunCommand(append([]string{"brew", "rmtree", "--dry-run"}, kws...))
	} else {
		err = RunCommand(append([]string{"brew", "rmtree"}, kws...))
	}

	errMsg := fmt.Sprintf("%s", err)
	if index := strings.Index(errMsg, "Unknown command: rmtree"); index != -1 {
		fmt.Printf(":: `rmtree` is not installed. You may try installing it with the following command:\n")
		fmt.Printf(":: brew tap beeftornado/rmtree\n")
		return
	}
	return
}

// S installs one or more packages by name.
func (pm *Homebrew) S(kws []string) (err error) {
	install := func(pack string) (err error) {
		code, err := pm.search(pack)
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

	for _, pack := range kws {
		if err = install(pack); err != nil {
			return
		}
	}

	return
}

// Sc removes all the cached packages that are not currently installed, and the unused sync database.
func (pm *Homebrew) Sc(kws []string) (err error) {
	if pm.DryRun {
		return RunCommand(append([]string{"brew", "cleanup", "--dry-run"}, kws...))
	}
	return RunCommand(append([]string{"brew", "cleanup"}, kws...))
}

// Scc removes all files from the cache.
func (pm *Homebrew) Scc(kws []string) (err error) {
	if pm.DryRun {
		return RunCommand(append([]string{"brew", "cleanup", "-s", "--dry-run"}, kws...))
	}
	return RunCommand(append([]string{"brew", "cleanup", "-s"}, kws...))
}

// Sccc ...
// ! What is this?
func (pm *Homebrew) Sccc(kws []string) (err error) {
	return NotImplemented()
}

// Sg lists all packages belonging to the GROUP.
func (pm *Homebrew) Sg(kws []string) (err error) {
	return NotImplemented()
}

// Si displays remote package information: name, version, description, etc.
func (pm *Homebrew) Si(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"brew", "info"}, kws...))
}

// Sii displays packages which require X to be installed, aka reverse dependencies.
func (pm *Homebrew) Sii(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"brew", "uses"}, kws...))
}

// Sl displays a list of all packages in all installation sources that are handled by the packages management.
func (pm *Homebrew) Sl(kws []string) (err error) {
	return NotImplemented()
}

// Ss searches for package(s) by searching the expression in name, description, short description.
func (pm *Homebrew) Ss(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"brew", "search"}, kws...))
}

// Su updates outdated packages.
func (pm *Homebrew) Su(kws []string) (err error) {
	if err = pm.RunIfNotDry(append([]string{"brew", "upgrade"}, kws...)); err != nil {
		return
	}
	return pm.RunIfNotDry(append([]string{"brew", "cask", "upgrade"}, kws...))
}

// Suy refreshes the local package database, then updates outdated packages.
func (pm *Homebrew) Suy(kws []string) (err error) {
	if err = pm.Sy(kws); err != nil {
		return
	}
	return pm.Su(kws)
}

// Sw retrieves all packages from the server, but does not install/upgrade anything.
func (pm *Homebrew) Sw(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"brew", "fetch"}, kws...))
}

// Sy refreshes the local package database.
func (pm *Homebrew) Sy(kws []string) (err error) {
	return pm.RunIfNotDry(append([]string{"brew", "update"}, kws...))
}

// U upgrades or adds package(s) to the system and installs the required dependencies from sync repositories.
func (pm *Homebrew) U(kws []string) (err error) {
	return NotImplemented()
}
