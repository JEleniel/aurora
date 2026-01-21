# Troubleshooting

## The command fails with exit code 1, but prints nothing

Some errors are only visible at higher log levels.

Run again with DEBUG logging:

```text
aurora_cli --log DEBUG <COMMAND>
```

## "Io error: No such file or directory (os error 2)" when rendering

This can happen when using `render-all` or `render-aurora` with clearing enabled and the output directory does not exist yet.

Work around it by creating the output directory first:

```text
mkdir -p tmp/aurora_cli_docs
```

Then re-run the render command.

## Validation reports schema errors

Schema and layout validation errors are reported with file and approximate line/column information.

Common causes:

- The card filename does not match `XXX-###.json`.
- The file stem does not match the JSON `id`.
- The folder name does not match the JSON `card_type`.
- A link target references an id that does not exist in the model.

## Views are missing from the per-mission README

The per-mission README only lists view links if the corresponding `*.view.md` files exist.

Use `render-all` (which renders views first) to ensure the view files exist before generating the README.
