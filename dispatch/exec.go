package dispatch

import (
	"fmt"
	"os"
	"os/exec"
	"strings"
)

// NotImplemented throws a "Not Implemented" error.
func NotImplemented() (err error) {
	return fmt.Errorf("Feature not implemented")
}

// PrintCommand prints the command to be executed.
func PrintCommand(cmd []string) {
	fmt.Printf(">> %s\n", strings.Join(cmd, " "))
}

// RunCommand and get the error.
func RunCommand(cmd []string) (err error) {
	PrintCommand(cmd)
	p := exec.Command(cmd[0], cmd[1:]...)
	p.Stdin = os.Stdin
	p.Stdout = os.Stdout
	p.Stderr = os.Stderr
	if err = p.Run(); err != nil {
		return fmt.Errorf("pacapt: error while running command `%s`", cmd)
	}
	return
}
