package dispatch

import (
	"os/exec"
	"runtime"
)

// PackManager represents a PACkage MANager
type PackManager interface {
	Q([]string) error
	Qc([]string) error
	Qe([]string) error
	Qi([]string) error
	Qk([]string) error
	Ql([]string) error
	Qm([]string) error
	Qo([]string) error
	Qp([]string) error
	Qs([]string) error
	Qu([]string) error
	R([]string) error
	Rn([]string) error
	Rns([]string) error
	Rs([]string) error
	S([]string) error
	Sc([]string) error
	Scc([]string) error
	Sccc([]string) error
	Sg([]string) error
	Si([]string) error
	Sii([]string) error
	Sl([]string) error
	Ss([]string) error
	Su([]string) error
	Suy([]string) error
	Sw([]string) error
	Sy([]string) error
	U([]string) error
}

// isExe checks if an executable exists by name (consult the PATH) or by path.
// To check by name (or path) only, pass the empty string as path (or name).
func isExe(name string, path string) bool {
	if len(name) != 0 {
		_, err := exec.LookPath(name)
		if err == nil {
			return true
		}
	}

	if len(path) != 0 {
		_, err := exec.LookPath(path)
		if err == nil {
			return true
		}
	}

	return false
}

// DetectPackManager detects the package manager in use.
// TODO: Make this function REALLY detect package managers
func DetectPackManager(dryRun bool, noConfirm bool) (pm PackManager) {
	pm = &Unknown{dryRun, noConfirm}

	switch runtime.GOOS {
	case "windows":
		// Chocolatey
		return &Chocolatey{dryRun, noConfirm}

	case "darwin":
		switch {
		// Homebrew
		case isExe("brew", "/usr/local/bin/brew"):
			return &Homebrew{dryRun}

		// Macports
		case isExe("port", "/opt/local/bin/port"):
			return // &Macports{dryRun, noConfirm}

		default:
			return
		}

	case "linux":
		switch {
		// Apt/Dpkg for Debian/Ubuntu/Termux
		case isExe("apt-get", "/usr/bin/apt-get"):
			return &Dpkg{dryRun, noConfirm}

		// Cave for Exherbo
		case isExe("cave", "/usr/bin/cave"):
			return // &Cave{dryRun, noConfirm}

		// Dnf for Red Hat
		case isExe("dnf", "/usr/bin/dnf"):
			return // &Dnf{dryRun, noConfirm}

		// Yum for Red Hat (Legacy)
		case isExe("yum", "/usr/bin/yum"):
			return // &Yum{dryRun, noConfirm}

		// Portage for Gentoo
		case isExe("emerge", "/usr/bin/emerge"):
			return // &Portage{dryRun, noConfirm}

		// Zypper for SUSE
		case isExe("zypper", "/usr/bin/zypper"):
			return // &Zypper{dryRun, noConfirm}

		// Apk for Alpine
		case isExe("apk", "/sbin/apk"):
			return // &Apk{dryRun, noConfirm}

		// Tazpkg for SliTaz
		case isExe("tazpkg", "/usr/bin/tazpkg"):
			return // &Tazpkg{dryRun, noConfirm}

		// Swupd for Clear Linux
		case isExe("swupd", "/usr/bin/swupd"):
			return // &Swupd{dryRun, noConfirm}

		default:
			return
		}

	case "freebsd":
		switch {
		// Pkgng
		case isExe("pkg", "/usr/sbin/pkg"):
			return // &Pkgng{dryRun, noConfirm}

		// PkgTools
		case isExe("pkg_add", "/usr/sbin/pkg_add"):
			return // &PkgTools{dryRun, noConfirm}

		default:
			return
		}

	case "openbsd":
		switch {
		// PkgTools
		case isExe("pkg_add", "/usr/sbin/pkg_add"):
			return // &PkgTools{dryRun, noConfirm}

		default:
			return
		}

	case "solaris":
		// SunTools
		return // SunTools{dryRun, noConfirm}

	default:
		return
	}
}
