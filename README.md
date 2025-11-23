# xortool-rs

A re-wite of [xortool.py](https://github.com/hellman/xortool) using Rust for
performance.

The original tool (xortool) is written in Python, and was MIT Licensed.

Supports:

* Guess the key length based on count of equal characters
* guess the key based on knowledge of the most frequent character

## Versioning Plans

Version 1.0.x executes the test script from the original xortool (v1.1.0) and
produces the same output, and the code will be visibly similar to the
original tools Python source

Version 1.x.x will contined to have the same output, but will have
re-working of the code to be more performant, and to leverage the Rust
langauge better.

