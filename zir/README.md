# ZIR

ZIR is a DSL created by the Risc0 team to support their zkvm. 

## Building zirgen 

To build with risc0's ZIR frontend (`zirgen`) first clone their repository and then build the tool. To build the tool you will need to install [Bazel](https://bazel.build/) version 6.0.0, which is the required version at the time of writing. If this version number is no longer correct the version declared in the `.bazelversion` file inside the repo prevails. An easy way to manage different bazel versions is with [Bazelisk](https://github.com/bazelbuild/bazelisk). 

```
git clone https://github.com/risc0/zirgen
cd zirgen 
# After installing Bazelisk, build zirgen like so
bazelisk build //zirgen/dsl:zirgen 
# The resulting binary should be located at bazel-bin/zirgen/dsl/zirgen
```

Alternatively the tool can be run directly with bazel, automatically building if necessary.

```
bazelisk run //zirgen/dsl:zirgen -- <zirgen arguments...>
```

Note that if running this way any path provided to `zirgen` must be absolute. For the rest of this documentation `zirgen` refers to whatever path or launching method is selected and paths will be relative, so change them as appropriate.

## Running tests inside the circuit 

`zirgen` has an integrated testing framework that allows running unit tests defined inside the circuit itself. To run the tests for the mastermind example and any other file it depends on run the following command.

```
zirgen mastermind.zir --test 
```
