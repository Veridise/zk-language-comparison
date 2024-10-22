# Mastermind (Lurk)

## Code Structure

The implementation in `mastermind.lurk` is self-contained. The first 48 lines implement the scoring function. These are
followed by some inline tests, in the form of assertions. Loading the file will exit with an error if an assertion
fails. The final lines create proofs-of-evaluation of the expressions invoking the scoring function. These can be
verified interactively from the Lurk REPL.

## Running the Examples

### Prerequisites
Install Lurk:
```
cargo install loam --locked --git https://github.com/argumentcomputer/lurk --rev 65d33a0
```

NOTE: it may be necessary to update rust (`rustup update`) in order for the installation to succeed.

### Command Line Instructions

```shell
lurk --preload mastermind.lurk
```

The command above will start Lurk's REPL and run `mastermind.lurk`.

To verify a proof, call the `verify` meta command and provide its key.

Example:
```lisp
!(verify "abcd1234...")
```
