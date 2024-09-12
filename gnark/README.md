# ZK Mastermind: Gnark

## Code Structure

- `mastermind.go`: Contains the implementation of the mastermind circuit and constraints.
- `mastermind_test.go`: Contains the setup and proving infrastructure as Go tests.
- `go.mod`: Contains the Go module definition and specifies dependencies.
- `go.sum`: Contains the checksums of the exact dependencies used.

## Running the Examples

### Prerequisites

Install the Go compiler ([go.dev instructions](https://go.dev/doc/install)).
You can also use a package manager for the installation, such as homebrew on
MacOS (see [here](https://formulae.brew.sh/formula/go) for `brew` instructions).

### Command Line Instructions

```sh
go test
```
