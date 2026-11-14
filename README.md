# vin

Initialise a research-paper workspace from a YAML directory template.

```console
$ vin list
Available templates (default: ai_ml)

  NAME           SOURCE     DIRS  FILES   DESCRIPTION
* ai_ml          built-in     62     15   AI / ML research: data, configs, experiments, results, reproduction
  econometrics   built-in     50     12   Empirical economics: chronology, estimation, robustness, replication
  physics        built-in     52     11   Physics: model, derivations, simulation, analysis

$ vin init my_paper -t ai --git
```

## Install

```sh
cargo install --path .
```

## Quick start

| Command | What it does |
|---------|--------------|
| `vin list` | List templates; the default is marked with `*` |
| `vin show <template> [-d N]` | Preview a template's tree |
| `vin init <destination> [-t template]` | Create a workspace; `.` = current directory (alias: `new`) |
| `vin help <command>` | Detailed help and examples (also `vin --help <command>`) |
| `vin default [template]` | Show or set the default template (`--reset` to clear) |
| `vin add <file.yaml>` | Install your own template |

Template names can be shortened to a unique prefix: `ai`, `econ`, `phys`.

`vin init` needs a destination: a folder name/path, or `.` for the current directory.

Useful `init` flags: `-C <dir>` (parent directory), `-n` (dry run), `-f` (overwrite), `--git` (run `git init`), `--no-gitkeep`.

## Documentation

See [`docs/`](docs/README.md):

- [Usage](docs/usage.md): every command and flag with examples
- [Templates](docs/templates.md): built-in templates and the YAML format
- [Configuration](docs/configuration.md): default template and custom templates
- [Architecture](docs/architecture.md): code layout and how to extend

## License

See [LICENSE](LICENSE).
