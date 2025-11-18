# xortool-rs

A re-wite of [xortool.py](https://github.com/hellman/xortool) using Rust for
performance.

The original tool (xortool) is written in Python, and was MIT Licensed.

Supports:

* Guess the key length based on count of equal characters
* guess the key based on knowledge of the most frequent character

## Versioning Plans

Version 1.0 executes the test script from the original xortool and
produces the same output, and the code will be visibly similar to the
original tools Python source

All later versions will focus on re-writing the code to better leverage
Rust, and to improve performance.
