[![license](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://github.com/clap-rs/clap/blob/master/LICENSE-APACHE)

# Cargo xbuild redirector

Allows cargo-xbuild to with with the IntelliJ ide (by replacing cargo in the xtensta toolchain 
with a version that checks for crosscompile, and call xbuild if necessary)

For example running:
```
cargo build
# will actually run xbuild...
```

Will trigger it to check the .cargo/config file for a ```target=xxx``` line. And if that line does not match your 
build architecture, it will re-invoke cargo with the same arguments, except that build will be replaced with xbuild:

```cargo xbuild```

While this is not useful for command line builds, it works great within IntelliJ, and this is only a temporary fix. As 
one day, cargo will be able to crosscompile without needing cargo-xbuild. 

# Installation

This assumes that you have created your own toolchain that does cross compiling, and will need to be installed
into that tool chain.

```bash
# Switch to toolchain where you want to cross compile.
$ rustup default xtensa
info: default toolchain set to 'xtensa'

# Run the install script to build and install.
./safer_install.sh
    cargo build....
    Finished release [optimized] target(s) in 0.01s
Moving "/home/yonasj/dev/tsim/cargo-xbuild-redirector/target/x86_64-unknown-linux-gnu/release/cargo-xbuild-redirector" to "/home/yonasj/.rustup/toolchains/xtensa/bin/cargo" to install re-director.
install complete.
```

After this, build in your IDE should work, though make sure you set the tool chain for your project:
```bash
$ rustup override set xtensa
info: override toolchain for '/home/yonasj/dev/tsim/cargo-xbuild-redirector' set to 'xtensa'
```
