# Helpman

Help man is a CLI tool to generate manpages from a rust binary's help
output.  This works well with clap output.  If there are subcommands,
their help gets included in the man page.

Usage: helpman [OPTIONS] <BINARY_PATH>

Arguments:

  <BINARY_PATH>  Path to the binary for which the manpage will be generated

Options:
  -n, --binary-name <BINARY_NAME>  Name of the binary (used in the manpage header). Defaults to the binary file name  
  -o, --output-dir <OUTPUT_DIR>    Directory where the generated manpage will be saved [default: .]  
  -s, --section <SECTION>          Section number of the manpage (accepted values: 1-8) [default: 1]  
  -t, --title <TITLE>              Title of the manual (default depends on the section)  
  -h, --help                       Print help  
  -V, --version                    Print version

## Install

Cargo install works:

```
cargo install --git https://github.com/jvasile/helpman.git --tag v0.1.0
```

The `--tag` parameter is optional.

Alternatively, if you are on a Debian system and have `cargo deb`
installed, you can `bin/dosh release` and then:

```
dpkg -i target/debian/helpman*.deb
```

## Dev helpers

 * bin contains dev utilities.  You might want to add it to your path temporarily while working on helpman.  I do this with direnv.

 * bin/dosh is a task runner

 * bin/cpsrc copies file contents to clipbboard

 * bin/helpman gets generated and will be a link to target/debug/helpman
