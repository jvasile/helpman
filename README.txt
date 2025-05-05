CLI tool to generate manpages from a binary's help output.  This
program handles subcommands by putting them all into one big man page.

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

Notes:

bin contains utilities.  You might want to add it to your path temporarily while working on helpman.  I do this with direnv.

bin/dosh is a task runner

bin/cpsrc copies file contents to clipbboard

bin/helpman gets generated and will be a link to target/debug/helpman
