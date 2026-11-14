# vin documentation

`vin` creates a ready-to-use directory structure for a research paper, from a YAML template. Three templates are built in (AI/ML, econometrics, physics) and you can add your own.

| Page | Contents |
|------|----------|
| [usage.md](usage.md) | Commands, flags, examples, exit behaviour |
| [templates.md](templates.md) | Built-in templates in detail; writing your own YAML |
| [configuration.md](configuration.md) | Config file, default template, custom template location |
| [architecture.md](architecture.md) | Source layout, design decisions, how to add a built-in template |

## 30-second tour

```sh
vin list                      # see what is available (default is marked *)
vin show physics -d 2         # preview a template two levels deep
vin default econometrics      # make it your default
vin init my_paper             # uses the default template
vin init other -t ai --git    # pick a template, run git init
vin init .                    # set up the current directory
vin help init                 # detailed help for any command
```
