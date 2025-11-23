#!/usr/bin/env pytest
# -*- coding: UTF-8 -*-
"""Smoke tests to run on xortool and compare results to the original Python xortool.

Note:
    This requires the Python package "xortool" to be installed in the same
    Python environment as the one running this file.

Note:
    This can take some time to run. It is recommended to use the
    pytest-xdist plugin to speed up running the tests.

"""

import subprocess
import os
import shutil
from pathlib import Path
import sys

import pytest


def hexdump_sim(path: str) -> str:
    """Simulate using hexdump and cut.

    This simulates the command
    ```hexdump -Cv $PATH | cut -s -d ' ' -f 3-20```

    Arguments:
        path: Path to the file to dump.

    Returns:
        Hexdump of the file.

    """
    with open(path, "rb") as file:
        contents = file.read()

    i = 0
    result = ""

    for byte in contents:
        if i == 7:
            end = "  "
        elif i == 15:
            end = " \n"
        else:
            end = " "
        result += f"{byte:02x}" + end
        i = (i + 1) % 16

    if i != 16:
        result += " " * (16 - i) + " \n"
    else:
        result += " \n"
    return result


def compare_dir_contents(dir1: Path, dir2: Path) -> None:
    """Compare contents of two directories.

    Arguments:
        dir1: First of the two directories to compare
        dir2: Second of the two directories to compare.

    """
    left_rel = {f.relative_to(dir1) for f in dir1.glob("**")}
    right_rel = {f.relative_to(dir2) for f in dir2.glob("**")}
    assert left_rel == right_rel
    for file in left_rel:
        if not file.is_dir():
            left_file = dir1 / file
            right_file = dir2 / file
            assert left_file.read_bytes() == right_file.read_bytes(), file


def run_rust_cmd(
    binary: str,
    args: list[str],
    stdin: bytes | None = None,
    workdir: Path | None = None,
) -> subprocess.CompletedProcess[bytes]:
    """Run a command using cargo.

    Arguments:
        binary: The name of the binary to run
        args: Arguments to pass to the binary
        stdin: Optional value to pass as standard input to the command
        workdir: Optional directory to run the command in.

    Returns:
        CompletedProcess object from running the command.

    """
    # Adding --Awarnings disables all warning messages from rust compiler
    # https://users.rust-lang.org/t/suppress-warnings-from-the-cargo-command/10536
    rustenv = os.environ.copy()
    rustenv["RUSTFLAGS"] = "-Awarnings"
    # -q disables any lines from cargo printing.
    # when combined with -Awarnings above, this results in the only
    # output being from the binary itself
    # cargo run --manifest-path ../Cargo.toml --bin xortool --release
    rust_cmd = [
        "cargo",
        "run",
        "-q",
        "--manifest-path",
        str(Path("Cargo.toml").resolve()),
        "--release",
        "--bin",
        binary,
        "--",
    ]
    rust_cmd.extend(args)

    # Run the rust tool
    return subprocess.run(
        rust_cmd,
        input=stdin,
        capture_output=True,
        env=rustenv,
        check=False,
        cwd=workdir,
    )


def compare_xortool_to_py(
    args: list[str],
    workdir: Path,
    stdin: str | bytes | None = None,
) -> None:
    """Run both Rust and Python xortool and compare the results.

    Arguments:
        command: The command to run
        args: The args to pass to the command.
        workdir: The directory to run the commands in.
        stdin: Option standard input to pass to the tools. If None,
            no standard input will be passed.

    """
    if isinstance(stdin, str):
        stdin = stdin.encode("utf-8")
    xortool_out = workdir / "xortool_out"
    xortool_out_rs = workdir / "xortool_out_rs"
    xortool_out_py = workdir / "xortool_out_py"

    rust_result = run_rust_cmd("xortool", args, stdin, workdir)
    if xortool_out.exists():
        shutil.move(xortool_out, xortool_out_rs)

    py_cmd = [sys.executable, "-m", "xortool.tool_main"]
    py_cmd.extend(args)
    # Run the python tool
    py_result = subprocess.run(
        py_cmd,
        input=stdin,
        capture_output=True,
        check=False,
        cwd=workdir,
    )

    # Check to ensure they produce the same outputs
    assert py_result.returncode == rust_result.returncode
    assert py_result.stdout == rust_result.stdout
    assert py_result.stderr == rust_result.stderr

    if xortool_out.exists():
        shutil.move(xortool_out, xortool_out_py)

    compare_dir_contents(xortool_out_py, xortool_out_rs)


def binary_xored_ok(check_dir: Path) -> bool:
    """Check the results from running on binary_xored are OK.

    Arguments:
        check_dir: Folder of output files to check

    Returns:
        Whether or not checks were successful.

    """
    if not check_dir.exists():
        return False
    result = True
    result = (
        result
        and "b'secret_key" in (check_dir / "filename-key.csv").read_text()
    )
    counts: dict[Path, int] = {}
    for file in check_dir.glob("*.out"):
        contents = file.read_bytes()
        counts[file] = contents.count(b"Free Software Foundation, Inc")
    counts = {k: v for k, v in counts.items() if v > 0}
    result = result and len(counts) == 1
    result = result and list(counts.values())[0] == 1
    return result


def ls_xored_ok(check_dir: Path) -> bool:
    """Check results from running on ls_xored_ok test file.

    Arguments:
        check_dir: Folder of output files to check.

    Returns:
        Whether or not checks were successful.

    """
    if not check_dir.exists():
        return False
    result = True
    result = (
        result
        and "b'really long s3cr3t k3y... PADDING'"
        in (check_dir / "filename-key.csv").read_text()
    )

    result = result and any(
        b"Free Software Foundation, Inc" in file.read_bytes()
        for file in check_dir.glob("*")
    )

    return result


def text_xored_ok(check_dir: Path) -> bool:
    """Check results from running on text_xored test file.

    Arguments:
        check_dir: Folder of output files to check.

    Returns:
        Whether or not checks were successful.

    """
    if not check_dir.exists():
        return False
    result = True
    result = (
        result
        and "b'\\xde\\xad\\xbe\\xef'"
        in (check_dir / "filename-key.csv").read_text()
    )
    result = result and any(
        b"List of known bugs" in file.read_bytes()
        for file in check_dir.glob("*")
    )
    return result


binary_xored_cases: list[tuple[list[str], str | None, bool]] = [
    (["--hex", "-b"], "test/data/binary_xored", True),
    (["-x", "-l", "10", "-c", "00"], "test/data/binary_xored", True),
    (["-x", "test/data/binary_xored"], None, False),
    (["--hex", "test/data/binary_xored"], None, False),
    (["-c", "00", "test/data/binary_xored"], None, True),
    (["--char=00", "test/data/binary_xored"], None, True),
    (["-b", "test/data/binary_xored"], None, True),
    (["-b", "-l", "10", "test/data/binary_xored"], None, True),
    (
        ["--brute-chars", "--key-length=10", "test/data/binary_xored"],
        None,
        True,
    ),
    (["-c", "00", "--key-length=10", "test/data/binary_xored"], None, True),
    (["-b", "--max-keylen=9", "test/data/binary_xored"], None, False),
    (["-b", "--key-length=16", "test/data/binary_xored"], None, False),
    (["-o", "test/data/binary_xored"], None, False),
    (["--brute-printable", "test/data/binary_xored"], None, False),
]

ls_xored_cases: list[tuple[list[str], str | None, bool]] = [
    (["--hex", "-b"], "test/data/ls_xored", True),
    (["-x", "-l", "33", "-c", "00"], "test/data/ls_xored", True),
    (["-x", "test/data/ls_xored"], None, False),
    (["--hex", "test/data/ls_xored"], None, False),
    (["-c", "00", "test/data/ls_xored"], None, True),
    (["--char=00", "test/data/ls_xored"], None, True),
    (["-b", "test/data/ls_xored"], None, True),
    (["-b", "-l", "33", "test/data/ls_xored"], None, True),
    (["--brute-chars", "--key-length=33", "test/data/ls_xored"], None, True),
    (["-c", "00", "--key-length=33", "test/data/ls_xored"], None, True),
    (["-b", "--max-keylen=32", "test/data/ls_xored"], None, False),
    (["-b", "--key-length=35", "test/data/ls_xored"], None, False),
    (["-o", "test/data/ls_xored"], None, False),
    (["--brute-printable", "test/data/ls_xored"], None, False),
]

text_xored_cases: list[tuple[list[str], bool]] = [
    (["-o", "test/data/text_xored"], True),
    (["-o", "-t", "printable", "test/data/text_xored"], True),
    (["-o", "-t", "base32", "test/data/text_xored"], True),
    (["-o", "-t", "base64", "test/data/text_xored"], True),
    (["-o", "-t", "a", "test/data/text_xored"], True),
    (["-o", "-t", "A", "test/data/text_xored"], True),
    (["-o", "-t", "1", "test/data/text_xored"], True),
    (["-o", "-t", "!", "test/data/text_xored"], True),
    (["-o", "-t", "*", "test/data/text_xored"], True),
    (["-o", "-t", "Z", "test/data/text_xored"], False),
    (["-o", "-t", "", "test/data/text_xored"], True),
]


@pytest.mark.parametrize("args,stdin,ok_func_result", binary_xored_cases)
def test_binary_xored(
    tmp_path: Path,
    args: list[str],
    stdin: str | None,
    ok_func_result: bool,
) -> None:
    """Test xortool on the binary_xored file, and compare output to original.

    Arguments:
        tmp_path: Pytest fixture to create a temporary directory to work
            in
        args: Arguments to pass to the tool
        stdin: Optional standard input to pass to the tools
            (i.e through a pipe)
        ok_func_result: Value that binary_xored_ok should return when
            run on the test's outputs.

    """
    if stdin:
        stdin = str(Path(stdin).resolve())
        compare_xortool_to_py(args, tmp_path, hexdump_sim(stdin))
    else:
        args[-1] = str(Path(args[-1]).resolve())
        compare_xortool_to_py(args, tmp_path, None)

    assert binary_xored_ok(tmp_path / "xortool_out_rs") == ok_func_result


@pytest.mark.parametrize("args,stdin,ok_func_result", ls_xored_cases)
def test_ls_xored(
    tmp_path: Path,
    args: list[str],
    stdin: str | None,
    ok_func_result: bool,
) -> None:
    """Test xortool on the ls_xored file, and compare output to original.

    Arguments:
        tmp_path: Pytest fixture to create a temporary directory to work
            in
        args: Arguments to pass to the tool
        stdin: Optional standard input to pass to the tools
            (i.e through a pipe)
        ok_func_result: Value that binary_xored_ok should return when
            run on the test's outputs.

    """
    if stdin:
        stdin = str(Path(stdin).resolve())
        compare_xortool_to_py(args, tmp_path, hexdump_sim(stdin))
    else:
        args[-1] = str(Path(args[-1]).resolve())
        compare_xortool_to_py(args, tmp_path, None)

    assert ls_xored_ok(tmp_path / "xortool_out_rs") == ok_func_result


@pytest.mark.parametrize("args,ok_func_result", text_xored_cases)
def test_text_xored(
    tmp_path: Path,
    args: list[str],
    ok_func_result: bool,
) -> None:
    """Test xortool on the text_xored file, and compare output to original.

    Arguments:
        tmp_path: Pytest fixture to create a temporary directory to work
            in
        args: Arguments to pass to the tool
        ok_func_result: Value that binary_xored_ok should return when
            run on the test's outputs.

    """
    args[-1] = str(Path(args[-1]).resolve())
    compare_xortool_to_py(args, tmp_path, None)
    assert text_xored_ok(tmp_path / "xortool_out_rs") == ok_func_result


def test_tool_xored(tmp_path: Path) -> None:
    """Test using the tool_xored file.

    Arguments:
        tmp_path: Pytest fixture to create a temporary directory to work
            in
    """
    compare_xortool_to_py(
        ["-o", str(Path("test/data/tool_xored").resolve())],
        tmp_path,
    )
    assert (
        "b'an0ther s3cret \\xdd key'"
        in (tmp_path / "xortool_out_rs/filename-key.csv").read_text()
    )
    assert any(
        b"# Author: hellman ( hellman1908@gmail.com )" in file.read_bytes()
        for file in (tmp_path / "xortool_out_rs").glob("*")
    )


def test_xortool_xor_1_2() -> None:
    """First and second of the tests of xortool-xor."""
    # This is the equivlent of the first two xortool_xor tests performed
    # by the original xortool's test.sh script
    stage1 = run_rust_cmd("xortool-xor", ["-n", "-s", "\\x3012345", "-r", "A"])
    assert stage1.returncode == 0
    assert stage1.stderr == b""
    assert stage1.stdout == b"qpsrut"
    stage2 = run_rust_cmd("xortool-xor", ["-r", "A", "-f-"], stage1.stdout)
    assert stage2.returncode == 0
    assert stage2.stderr == b""
    assert stage2.stdout == b"012345\n"


def test_xortool_xor_3() -> None:
    """Third of the tests of xortool-xor."""
    # This is the equivlent of the third xortool_xor test in the original xortool's test.sh script
    stage1 = run_rust_cmd(
        "xortool-xor", ["-n", "-h", "30 31 32 33 34  35  ", "-r", "A"]
    )
    assert stage1.returncode == 0
    assert stage1.stderr == b""
    assert stage1.stdout == b"qpsrut"
    stage2 = run_rust_cmd("xortool-xor", ["-r", "A", "-f-"], stage1.stdout)
    assert stage2.returncode == 0
    assert stage2.stderr == b""
    assert stage2.stdout == b"012345\n"


def test_xortool_xor_4() -> None:
    """Fourth of the tests of xortool-xor."""
    # This is the equivlent of the third xortool_xor test in the original xortool's test.sh script
    cmdout = run_rust_cmd("xortool-xor", ["-n", "-r", "qpsrut", "-r", "A"])
    assert cmdout.returncode == 0
    assert cmdout.stderr == b""
    assert cmdout.stdout == b"012345"


# FIXME: Need to do the last bit of testing in the original test.sh
# comes after the xortool_xor tests, and combines the two


def test_combo(tmp_path: Path) -> None:
    """Test a combination of using both xortool and xortool-xor.

    Arguments:
        tmp_path: Pytest fixture to create a temp folder to run in.

    """
    compare_xortool_to_py(
        [
            "-c",
            "00",
            "--key-length=10",
            str(Path("test/data/binary_xored").resolve()),
        ],
        tmp_path,
    )
    matching_file = ""
    with (tmp_path / "xortool_out_rs/filename-key.csv").open() as file:
        for line in file:
            if "secret_key" in line:
                matching_file = line.split(";")[0]
    assert matching_file
    full_path = tmp_path / matching_file.replace(
        "xortool_out", "xortool_out_rs"
    )
    xortool_xor_1 = run_rust_cmd(
        "xortool-xor",
        ["-n", "-r", "secret_key", "-f", str(full_path.resolve())],
    )
    assert xortool_xor_1.returncode == 0
    assert xortool_xor_1.stderr == b""
    assert xortool_xor_1.stdout == Path("test/data/binary_xored").read_bytes()

    xortool_xor_2 = run_rust_cmd(
        "xortool-xor",
        [
            "-n",
            "-r",
            "secret_key",
            "-f",
            str(Path("test/data/binary_xored").resolve()),
        ],
    )
    assert xortool_xor_2.returncode == 0
    assert xortool_xor_2.stderr == b""
    assert xortool_xor_2.stdout == full_path.resolve().read_bytes()
