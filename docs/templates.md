# Templates

## Built-in templates

All three share a common core: `data`, `sources`, `results`, `literature` (`prose`, `references`), `latex`, `reports`, `dport`, `submission`, `template`, `docs`, plus starter files (`README.md`, `.gitignore`, `Makefile`).

### `ai_ml` (default)

AI/ML research papers.

| Directory | Purpose |
|-----------|---------|
| `data` | `raw` (immutable), `interim`, `processed`, `external`, `splits`, `DATA_CARD.md` |
| `configs` | model, training, experiment and sweep configs |
| `sources` | project code (`src`, `tests`), preserved from the working repo |
| `notebooks` | exploration and analysis |
| `experiments` | one folder per run, with `EXPERIMENT_LOG.md` |
| `results` | metrics, tables, figures, checkpoints, ablations, logs, analysis |
| `diagram` | plot/diagram sources and exports |
| `latex` | sections, figures, tables, bibliography, appendix, supplementary, `main.tex` |
| `literature` | `prose`, `references` (`references.bib`), `pdfs`, `notes`, `reading_list.md` |
| `reproduction` | `repos`, `logs`, `REPRODUCTION_NOTES.md` |
| `reports`, `dport`, `presentation`, `submission`, `template`, `docs`, `scripts` | communication, delivery, tooling |

Also: `requirements.txt`, `environment.yml`, a submission `checklist.md`.

### `econometrics`

Empirical economics.

Distinctive parts: `chronology` (timeline, work log), `data/codebook`, `sources/{cleaning,estimation,robustness}`, `results/{tables,diagnostics,robustness,descriptive}`, `visualization/{charts,diagram,maps}`, `replication` (journal replication package), `docs/identification_strategy.md`, and `submission/{cover_letter,response_to_referees}`.

### `physics`

Theory, simulation or experiment.

Distinctive parts: `model` (assumptions, parameters and units, notation), `derivations`, `simulation/{src,inputs,runs,convergence}` with `SIM_LOG.md`, `analysis/{fitting,error_analysis}`, `diagram/schematics`, `send/{collaborators,referees}`, and a physics-oriented `main.tex` preamble (`amsmath`, `physics`, `siunitx`).

Use `vin show <name>` for the exact tree.

## YAML format

A template is a YAML file with **one root key**. The value types decide what is created:

| YAML value | Creates |
|------------|---------|
| mapping | a directory containing the entries |
| `null` (or nothing after the colon) | an empty directory |
| string | a file with that content (`""` = empty file) |

```yaml
# description: My lab's layout          # optional, shown by `vin list`
my_template:                            # root key (default folder name)
  README.md: |
    # <Paper Title>                     # replaced by the workspace name
  data:
    raw:                                # empty directory (gets .gitkeep)
    processed:
  notes.md: ""                          # empty file
```

Rules:

- The root key must be a directory; there must be exactly one.
- Keys must be strings and must not be empty, `.`/`..`, or contain `/` or `\`.
- Duplicate names in the same directory are an error.
- Numbers, booleans and lists are rejected: quote file contents (`"5"`) or use a block scalar (`|`).
- Block scalars preserve tabs. Makefile recipes need real tab characters.
- Order is preserved, which affects `show` output only.

## Writing your own

1. Copy a built-in as a starting point: `src/directories_structure/ai_ml.yaml`.
2. Edit it; add a `# description: ...` comment line for `vin list`.
3. Install: `vin add mytemplate.yaml` (validates first).
4. Use: `vin init paper1 -t mytemplate`, or `vin default mytemplate`.

Custom templates are stored in the [config directory](configuration.md) and can also be placed there by hand as `<name>.yaml`.
