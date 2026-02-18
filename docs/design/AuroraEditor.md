# Standalone, Cross Platform Editor Specifications

## Requirements

### Platform & Loading

- Runs on the major desktop platforms: Linux, Microsoft Windows, and Apple macOS.
- Can load and work with a model of any size or complexity.
- Models of any size load almost instantly. The load method traverses and validates the model in the time it takes to read files; the editor does not load the entire model into memory at once.
- Loads the schemas and reference files included with a set of models in a model home and uses those for all interaction with those models. This allows the editor to work across multiple versions and customizations of Aurora.
- Makes a backup ZIP of the model at load time, stored per conventions defined in [Aurora_Specs/](../../Aurora_Specs/).
- Storage location and format for models and configuration are defined in [Aurora_Specs/](../../Aurora_Specs/).

### Architecture & Performance

- UI and engine operate on separate threads to ensure responsiveness and take full advantage of modern hardware.
- Memory constraints: TBD (to be collected during development).

### Features & Editing

- Navigate and understand complex architectures through a mind-map like centered element views with related links and dependencies displayed at a glance.
- Can render any view defined in the model configuration file and ad-hoc views.
- Can pack and unpack the model in a single ZIP-compressed file.
- Guaranteed to prevent edits that would break a model. Also provides style checking and linting.
- Uses an underlying library shared across tools, giving it all the capabilities of the CLI.
- Immediate autosave (Apple style) by default, with an option for manual save mode.
- Undo/redo spanning 50 edits deep (current target; TBD pending testing and memory constraints).

### Data Integrity & Safety

- With autosave enabled: on a crash, the model may at worst contain an orphan card that needs to be linked; cards are written in a single operational call.
- With autosave disabled: the saved model is always valid; unsaved changes are lost on crash.
- User confirmation required before closing or exiting when unsaved changes exist.
- Multiple instances on the same model are not supported; last write wins if concurrent edits occur. Shared/networked models are not officially supported; collaborative editing is a future roadmap item.

### Persistence & Storage

See [Aurora_Specs/](../../Aurora_Specs/) for detailed storage format and location specifications.

### Usability & Accessibility

- WCAG AA compliant accessibility.
- Dark mode support enabled; dark mode is the default.
- Adjustable base font size; all other UI elements scale relative to this setting.

### Logging

- Logging required with fern integration.
- Support for stdout, stderr, and optional file-based logging.

### Compatibility

- Each model home includes a complete set of schema and configuration files snapshotted at model creation time.
- Compatibility is guaranteed across a major version of Aurora (the system is separately versioned); the appropriate schema and configuration versions are always available within the model home.
- Incompatible models are detected via schema validation (behavior defined in Aurora_Specs/).

## Future Ideas

- **Unlimited undo/redo:** Full changelog storage would enable unlimited undo/redo depth beyond the current 50-edit limit.
- **Access Control & RBAC:** Role-based permissions, read/write controls, and audit logging.
- **Extensible Plugin API:** Allow third-party extensions for importers, exporters, visualizations, and automation.
- **Collaborative Editing:** Real-time collaboration (multi-user) with conflict resolution and presence indicators.
- **Update/Upgrade Tooling:** Mechanisms for upgrading model homes to new Aurora versions.

## Technologies

- Rust 2024
- Dioxus
