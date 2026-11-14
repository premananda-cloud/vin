# Architecture

```
src/
├── main.rs        CLI definition (clap) and command handlers
├── config.rs      config dir, config.toml, default template
├── template.rs    built-in (embedded) + custom template discovery, name resolution
├── tree.rs        YAML -> Node tree, validation, stats, ASCII rendering
├── init.rs        write a tree to disk (dry-run, force, .gitkeep, title placeholder)
└── directories_structure/
    ├── ai_ml.yaml
    ├── econometrics.yaml
    └── physics.yaml
install.sh             end-user installer (prebuilt binary, source fallback)
scripts/release.sh     bump version, check, commit, tag, push
.github/workflows/
├── ci.yml             fmt, clippy, tests, shellcheck on every push/PR
└── release.yml        on `v*` tag: build 4 targets, publish release
```

## Data flow

```
YAML text --tree::parse--> Tree { root_name, children: Vec<(String, Node)> }
                                  |
          list/show: stats + render        init: init::materialize(base, children, ...)
```

`Node` is either `Dir(Vec<(String, Node)>)` or `File(String)`. Children are a `Vec` (not a map) to preserve the YAML order.

## Design decisions

- **Templates are embedded** with `include_str!`, so the binary is self-contained and needs no data files at runtime.
- **Custom templates override built-ins by name**, so you can adapt a built-in without forking the program.
- **Validation happens at parse time** (`tree::parse`): path traversal, duplicate names, non-string content. `init` never sees an invalid tree.
- **`init` never overwrites by default.** Existing files are skipped and counted, which makes re-running safe.
- **Unique-prefix matching** keeps the CLI short (`-t phys`) without a separate alias table.
- **`serde_yaml` 0.9** is used. The crate is marked deprecated upstream but is stable and sufficient here; `serde_yaml_ng` is a drop-in alternative if you prefer a maintained fork.

## Adding a built-in template

1. Add `src/directories_structure/<name>.yaml` (see [templates.md](templates.md)).
2. Register it in the `BUILTIN` table in `src/template.rs`:

   ```rust
   ("chemistry", "Chemistry: ...", include_str!("directories_structure/chemistry.yaml")),
   ```

3. `cargo test`: `builtins_all_parse` validates it automatically.

## Tests

```sh
cargo test
```

Unit tests cover YAML parsing and validation, prefix resolution, and writing to disk (creation, `.gitkeep`, title substitution, no-overwrite, `--force`, dry run) using temporary directories.
