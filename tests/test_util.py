from typing import List, Optional, Tuple
import re
import subprocess
from io import StringIO

EXE: List[str] = ["cargo", "run", "--"]


class Test:
    sequence: List[Tuple[List[str], List[str]]]
    cmd: Optional[List[str]]

    def __init__(self):
        self.sequence = list()
        self.cmd = None

    def input(self, cmd: List[str]) -> "Test":
        # Guard against self.input().input().
        if self.cmd is not None:
            raise RuntimeError("Unexpected consecutive input")
        self.cmd = cmd
        return self

    def output(self, out: List[str]) -> "Test":
        # Guard against self.output() without self.cmd being set.
        if self.cmd:
            self.sequence.append((list(self.cmd), out))
        else:
            raise RuntimeError("Expected an input before an output")
        # Reset self.cmd.
        self.cmd = None
        return self

    def run(self, verbose: bool = False) -> None:
        def matches_all(out: str, patterns: List[str]) -> bool:
            return all(re.compile(p).search(out) for p in patterns)

        # Prevent running the test before a sequence is configured.
        if not self.sequence:
            raise RuntimeError("Test sequence not yet configured")

        # Then we should run the command and check against `self.sequence`.
        for cmd, patterns in self.sequence:
            # got = cmd.run()
            # if not matches_all(got, patterns):
            #     raise BlablaError(some_msg)
            got: str = ""
            with subprocess.Popen(
                EXE+cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                bufsize=1,
                universal_newlines=True
            ) as p, StringIO() as buf:
                if p.stdout is None:
                    raise RuntimeError("Popen failed to capture output")
                for line in p.stdout:
                    print(line, end='')
                    buf.write(line)
                got = buf.getvalue()
            # code = p.returncode
            if not matches_all(got, patterns):
                raise RuntimeWarning(f"Failed with {EXE+cmd}")
