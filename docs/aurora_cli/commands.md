# Command reference

The CLI command name is `aurora_cli`.

## Global options

| Option | Description | Default |
| --- | --- | --- |
| `-i, --input <INPUT_PATH>` | Path to an Aurora model home, its parent folder, or the `aurora/` folder itself. | `docs/design/aurora/` |
| `-l, --log <LOG_LEVEL>` | Log level used by the CLI. | `INFO` |

`LOG_LEVEL` is parsed as a Rust `log::Level` (for example: `ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`).

## validate

Validates schemas and model invariants.

```text
aurora_cli validate
```

## render-aurora

Renders card Markdown outputs and a per-mission README.

```text
aurora_cli render-aurora --output <OUTPUT_PATH>
```

Options:

| Option | Description | Default |
| --- | --- | --- |
| `-o, --output <OUTPUT_PATH>` | Output root for generated Markdown. | `docs/design/` |
| `-c, --clear` | Clear existing `.md` files under the output root before writing. | Enabled by default |

## render-views

Renders view Markdown outputs (`*.view.md`) for each model.

```text
aurora_cli render-views --output <OUTPUT_PATH>
```

Options:

| Option | Description | Default |
| --- | --- | --- |
| `-o, --output <OUTPUT_PATH>` | Output root for generated Markdown. | `docs/design/` |
| `-c, --clear` | Accepted by the CLI, but view rendering does not currently clear outputs. | N/A |

## render-all

Renders views first, then card Markdown.

```text
aurora_cli render-all --output <OUTPUT_PATH>
```

Options:

| Option | Description | Default |
| --- | --- | --- |
| `-o, --output <OUTPUT_PATH>` | Output root for generated Markdown. | `docs/design/` |
| `-c, --clear` | Clear existing `.md` files under the output root once, then render. | Enabled by default |

## compact

Writes an agent-friendly JSON representation of each model.

```text
aurora_cli compact --output <OUTPUT_PATH>
```

The compact model file is written as `AGENT-<MISSION_ID>.json` in the output directory.

## bump-patch / bump-minor / bump-major

Bumps the version in a card's `audit_trail` and appends an audit history entry.

```text
aurora_cli bump-patch --card <CARD_ID>
aurora_cli bump-minor --card <CARD_ID>
aurora_cli bump-major --card <CARD_ID>
```

Options:

| Option | Description |
| --- | --- |
| `-c, --card <CARD_ID>` | The card id to bump (for example: `REQ-001`). |
| `-e, --editor <NAME>` | Override the audit history editor value. |

Warning: bump commands modify the source JSON card file in-place. Run them against a clean working tree and review changes before committing.
