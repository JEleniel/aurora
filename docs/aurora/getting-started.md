# Getting started

This guide walks you through creating and validating your first Aurora model.

## Navigation

- [Aurora overview](README.md)
- [Model and folder layout](model-layout.md)
- [aurora_cli Quickstart](../aurora_cli/quickstart.md)
- [Design docs index](../design/README.md)

## 1. Create a model home

Create a folder named `aurora/` (or any folder you want to use as the model home), and add the schemas:

- `Aurora.schema.json`
- `Aurora.compact.schema.json`

In this repository, the design model is under:

- `docs/design/aurora/`

## 2. Add a Mission card

Create a Mission JSON file in the model home:

```text
MIS-001-My_First_Model.json
```

And create a folder for the mission id:

```text
MIS-001/
```

## 3. Add a few cards

Add at least one path away from the mission.

Example:

```text
MIS-001/Driver/DRI-001.json
MIS-001/Requirement/REQ-001.json
```

## 4. Validate

Validate the model:

```text
aurora_cli --input <MODEL_HOME> validate
```

Sample output:

```text
All models are valid.
```

## 5. Render documentation

Create an output folder and render cards and views:

```text
mkdir -p tmp/aurora_docs
```

Sample output:

```text
(no output)
```

Then:

```text
aurora_cli --input <MODEL_HOME> render-all --output tmp/aurora_docs
```

Sample output:

```text
Rendered views for model MIS-001.
Rendered cards for 1 models.
```

## Next steps

- Expand the graph: add Capabilities, Features, Systems, Applications, and Components.
- Add Behavior: Processes and State Machines.
- Add Assurance: Controls, Risks, Threats, and Tests.
