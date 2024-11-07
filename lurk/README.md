# Mastermind (Lurk)

## Code Structure

The implementation in `mastermind.lurk` is self-contained. The first 48 lines implement the scoring function. These are
followed by some inline tests, in the form of assertions. Loading the file will exit with an error if an assertion
fails. The final lines create proofs-of-evaluation of the expressions invoking the scoring function and then verify
those proofs.

## Running the Examples

### Prerequisites
Install Lurk:
```
cargo install lurk --locked --git https://github.com/argumentcomputer/lurk --rev 3c7e883
```

NOTE: it may be necessary to update rust (`rustup update`) in order for the installation to succeed.

### Command Line Instructions

```shell
lurk mastermind.lurk
```
