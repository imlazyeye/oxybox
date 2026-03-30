`oxybox`

Bindings to the box2D library.

## Binding to the C Code

We get the source code of box2d using git submodules.
We bind to that source code using the bindgen cli.

You can easily do this yourself by running `bindgen.sh`, which will
update the submodule and run bindgen for you. Note: this requires an exact
version of bindgen. You may need to install the version required with
`cargo install bindgen --verison X`. The shell script itself will
error out and give the required version if you do not have it.