yeet-repo
======
Yeet a git repo into a text file.

```console
$ yeet-repo --help
Convert a Git repository to a text file.

USAGE:
    yeet-repo [OPTIONS] <REPO_PATH>

ARGS:
    <REPO_PATH>    The path to the Git repository

OPTIONS:
    -h, --help                   Print help information
    -i, --ignore <IGNORE>        The path to a custom ignore file. If not given, tries to fallback
                                 to `{repo_path}/.gptignore`
    -o, --output <OUTPUT>        The path to the output file [default: output.txt]
    -p, --preamble <PREAMBLE>    The path to the preamble file. If not given, uses a sensible
                                 default
    -r, --stderr                 Print repository contents to stderr
    -s, --stdout                 Print repository contents to stdout

```
