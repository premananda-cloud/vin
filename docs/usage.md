# Usage

```
vin <COMMAND>

Commands:
  init     Create a new paper workspace from a template   (alias: new)
  list     List available templates                        (alias: ls)
  show     Preview a template's directory tree
  default  Show or change the default template
  add      Install a custom template from a YAML file
```

Running `vin` with no command prints help.

## Getting help

Every command has detailed help with examples:

```sh
vin help init        # or: vin --help init, vin init --help
vin help show
vin help default
vin help add
vin help list
```

`vin --help` shows the overview, and `vin help <command>` shows the options and examples for one command.

## Choosing a template

Everywhere a template is accepted you can use its full name or any **unique prefix**:

| You type | Resolves to |
|----------|-------------|
| `ai`, `ai_ml` | `ai_ml` |
| `econ` | `econometrics` |
| `phys` | `physics` |

An ambiguous prefix is an error that lists the matches. When no template is given, the **default template** is used (see [configuration](configuration.md)); out of the box that is `ai_ml`.

## `vin list`

Shows every template with its source (`built-in` or `custom`), directory and file counts, and a description. The default is marked with `*`. The footer shows where custom templates are stored.

## `vin show [TEMPLATE] [-d DEPTH]`

Prints the template as a tree without creating anything. `-d 1` shows only the top level; omitted shows everything.

```console
$ vin show econ -d 1
paper_template_for_econometrics/
├── README.md
├── chronology/
│   └── …
...
```

## `vin init <DESTINATION> [OPTIONS]`

Creates the workspace. A **destination is required**; without one, vin stops with `please provide a destination`.

| Argument / flag | Meaning |
|-----------------|---------|
| `DESTINATION` | Required. A folder name or path to create (`my_paper`, `~/papers/wage_gap`), or `.` to set up the **current directory**. |
| `-t, --template <T>` | Template name or prefix. Defaults to the default template. |
| `-C, --path <DIR>` | Create the workspace inside `DIR` instead of the current directory. |
| `-n, --dry-run` | Print the summary, write nothing. |
| `-f, --force` | Overwrite files that already exist. |
| `--no-gitkeep` | Do not add `.gitkeep` to empty directories. |
| `--git` | Run `git init` in the new workspace. |

Behaviour:

- **Existing directories are merged into.** Existing files are **skipped** and reported, never overwritten, unless `--force` is given. Re-running `init` is therefore safe.
- **Empty directories get a `.gitkeep`** so git tracks the structure. The generated `.gitignore` allows it (`!**/.gitkeep`).
- **`<Paper Title>`** inside template files is replaced with the name of the destination folder (for `.`, the current folder's name).
- Missing parent folders in a path are created (`vin init a/b/c` works).

### Examples

```sh
vin init attention_study -t ai --git       # new folder, AI/ML template, git repo
vin init . -t physics                      # populate the current (empty) folder
vin init wage_gap -t econ -C ~/papers      # ~/papers/wage_gap
vin init demo -t ai -n                     # dry run
vin init demo -t ai --force                # refresh template files in place
vin init                                   # error: please provide a destination
```

## `vin default [TEMPLATE] [--reset]`

- `vin default` prints the current default.
- `vin default physics` sets it (stored in the config file).
- `vin default --reset` returns to the built-in default (`ai_ml`).

## `vin add FILE [--name NAME] [--force]`

Validates a template YAML and installs it as a custom template. The name defaults to the file name; it may contain letters, digits, `_` and `-`. A custom template with the same name as a built-in one **overrides** it. See [templates.md](templates.md) for the format.

## Errors

Errors are printed to stderr as `error: ...` and the exit code is `1`. Typical causes: unknown or ambiguous template, invalid YAML, unwritable target.
