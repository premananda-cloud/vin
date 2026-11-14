# Configuration

vin stores settings and custom templates in one directory:

| Platform | Default location |
|----------|------------------|
| Linux | `~/.config/vin/` |
| macOS | `~/Library/Application Support/vin/` |
| Windows | `%APPDATA%\vin\` |

Set `VIN_CONFIG_DIR` to use a different directory (handy for testing or per-project setups).

```
<config dir>/
├── config.toml          # settings
└── templates/           # custom templates
    └── mylab.yaml
```

## `config.toml`

```toml
default_template = "physics"
```

| Key | Meaning |
|-----|---------|
| `default_template` | Template used when `-t` is omitted. If absent, `ai_ml`. |

Normally you never edit this by hand: use `vin default <name>` / `vin default --reset`. If the configured default no longer exists, `vin list` prints a warning.

## Custom templates

Any `*.yaml` / `*.yml` file in `templates/` is picked up automatically; the file name (without extension) is the template name. A custom template whose name matches a built-in **replaces** it. Remove a custom template by deleting its file.

`vin add` is the safe way to install one: it validates the YAML first.

## Environment variables

| Variable | Effect |
|----------|--------|
| `VIN_CONFIG_DIR` | Override the config directory |
| `NO_COLOR` | Disable coloured output (colour is also off when output is piped) |
