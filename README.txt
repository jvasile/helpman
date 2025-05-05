CLI tool to generate manpages from a binary's help output

Usage: helpman [OPTIONS] <BINARY_PATH>

Arguments:
  <BINARY_PATH>  Path to the binary for which the manpage will be generated

Options:
  -n, --binary-name <BINARY_NAME>  Name of the binary (used in the manpage header). Defaults to the binary file name
  -o, --output-dir <OUTPUT_DIR>    Directory where the generated manpage will be saved (defaults to current working directory) [default: .]
  -s, --section <SECTION>          Section number of the manpage (default is 1, accepted values: 1-8) [default: 1]
  -t, --title <TITLE>              Title of the manual (default depends on the section)
  -h, --help                       Print help
  -V, --version                    Print version
