#!/usr/bin/env python3
# -*- coding: UTF-8 -*-
import sys
from argparse import ArgumentParser
from pathlib import Path
from shutil import copy2, make_archive


def main() -> None:
    """Bundle binaries into a zip for release."""
    parser = ArgumentParser()
    parser.add_argument(
        "srcdir",
        type=Path,
        help="Path that the binaries can be found in",
    )
    parser.add_argument(
        "dst",
        type=Path,
        help="Name of the zip file to place binaries in.",
    )
    parser.add_argument(
        "files",
        nargs="+",
        help="The files to bundle into the zip.",
    )

    args = parser.parse_args()

    tmpdir = Path("tmp")
    tmpdir.mkdir()

    for file in args.files:
        path: Path = args.srcdir / file
        if sys.platform == "win32":
            path = path.with_suffix(".exe")
        copy2(path, tmpdir / path.name)

    make_archive(args.dst, "zip", tmpdir)


if __name__ == "__main__":
    main()
