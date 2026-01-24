# MIS-001: Provide Default Tooling for AURORA - Component

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR


	APP-001["`**Application**: APP-001<br />Command Line Tools`"]@{shape: lin-rect}
	APP-002["`**Application**: APP-002<br />Editor`"]@{shape: lin-rect}
	APP-003["`**Application**: APP-003<br />VSCode Extension`"]@{shape: lin-rect}
	ART-001["`**Artifact**: ART-001<br />Aurora Schema`"]@{shape: documents}
	ART-002["`**Artifact**: ART-002<br />Compact Schema`"]@{shape: documents}
	ART-003["`**Artifact**: ART-003<br />Model Files`"]@{shape: documents}
	ART-004["`**Artifact**: ART-004<br />Rendered Cards`"]@{shape: documents}
	ART-005["`**Artifact**: ART-005<br />Rendered Views`"]@{shape: documents}
	ART-006["`**Artifact**: ART-006<br />Compact Model Export`"]@{shape: documents}
	ART-007["`**Artifact**: ART-007<br />Validation Diagnostics`"]@{shape: documents}
	ART-008["`**Artifact**: ART-008<br />Preview Content`"]@{shape: documents}
	COM-001[["`**Component**: COM-001<br />Shared Library`"]]
	COM-002[["`**Component**: COM-002<br />Tauri Svelte UI`"]]
	COM-003[["`**Component**: COM-003<br />Tauri Rust Backend`"]]
	COM-004[["`**Component**: COM-004<br />Explorer Tree View`"]]
	COM-005[["`**Component**: COM-005<br />Mind-Map Like View`"]]
	COM-006[["`**Component**: COM-006<br />Edit View`"]]
	COM-007[["`**Component**: COM-007<br />Markdown View`"]]
	COM-008[["`**Component**: COM-008<br />MCP Server`"]]
	COM-009[["`**Component**: COM-009<br />Aurora`"]]
	COM-010[["`**Component**: COM-010<br />Model`"]]
	COM-011[["`**Component**: COM-011<br />Card`"]]
	COM-012[["`**Component**: COM-012<br />Aurora CLI`"]]
	COM-013[["`**Component**: COM-013<br />VSCode Extension Host`"]]
	COM-014[["`**Component**: COM-014<br />Webview UI Host`"]]
	COM-015[["`**Component**: COM-015<br />Workspace Adapter`"]]
	DTS-001[("`**Data Store**: DTS-001<br />Workspace File System`")]
	DTS-002[("`**Data Store**: DTS-002<br />Extension State Store`")]
	INT-001["`"`**Interface**: INT-001<br />CLI Interface`"`"]@{shape: delay}
	INT-002["`"`**Interface**: INT-002<br />MCP Interface`"`"]@{shape: delay}
	INT-003["`"`**Interface**: INT-003<br />Editor Backend API`"`"]@{shape: delay}
	INT-004["`"`**Interface**: INT-004<br />VSCode Webview Messaging API`"`"]@{shape: delay}
	INT-005["`"`**Interface**: INT-005<br />VSCode Commands Interface`"`"]@{shape: delay}
	NOT-001["`**Note**: NOT-001<br />Shareable`"]@{shape: braces}
	SYS-001["`"`**System**: SYS-001<br />Default Tooling Platform`"`"]@{shape: div-rect}
	TES-001{{"`**Test**: TES-001<br />Validate Model Command`"}}
	TES-002{{"`**Test**: TES-002<br />Render All Command`"}}
	TES-003{{"`**Test**: TES-003<br />Compact Command`"}}


	APP-001 -- includes --> COM-001;
	APP-001 -- includes --> COM-012;
	APP-002 -- includes --> COM-001;
	APP-002 -- includes --> COM-002;
	APP-002 -- includes --> COM-003;
	APP-002 -- includes --> COM-004;
	APP-002 -- includes --> COM-005;
	APP-002 -- includes --> COM-006;
	APP-002 -- includes --> COM-007;
	APP-003 -- includes --> COM-001;
	APP-003 -- includes --> COM-008;
	APP-003 -- includes --> COM-013;
	APP-003 -- includes --> COM-014;
	APP-003 -- includes --> COM-015;
	APP-003 -- includes --> COM-004;
	APP-003 -- includes --> COM-005;
	APP-003 -- includes --> COM-006;
	APP-003 -- includes --> COM-007;
	APP-003 -- uses --> NOT-001;
	COM-001 -- includes --> COM-009;
	COM-001 -- includes --> COM-010;
	COM-001 -- includes --> COM-011;
	COM-002 -- uses --> COM-003;
	COM-003 -- uses --> COM-001;
	COM-003 -- exposes --> INT-003;
	COM-008 -- uses --> COM-001;
	COM-008 -- exposes --> INT-002;
	COM-009 -- comprises --> COM-010;
	COM-010 -- comprises --> COM-011;
	COM-012 -- uses --> COM-001;
	COM-012 -- exposes --> INT-001;
	COM-013 -- uses --> COM-008;
	COM-013 -- uses --> COM-015;
	COM-013 -- exposes --> INT-004;
	COM-013 -- exposes --> INT-005;
	COM-014 -- uses --> COM-013;
	COM-015 -- uses --> COM-001;
	COM-015 -- persists_to --> DTS-001;
	COM-015 -- persists_to --> DTS-002;
	SYS-001 -- integrates --> APP-001;
	SYS-001 -- integrates --> APP-002;
	SYS-001 -- integrates --> APP-003;


classDef cls_boundary stroke-dasharray:5 5,stroke-width:4;
classDef cls_mission fill:#022c22,color:#FFFFFF
classDef cls_driver fill:#064e3b,color:#FFFFFF
classDef cls_requirement fill:#065f46,color:#FFFFFF
classDef cls_capability fill:#052e16,color:#FFFFFF
classDef cls_feature fill:#14532d,color:#FFFFFF
classDef cls_actor fill:#1a2e05,color:#FFFFFF
classDef cls_story fill:#365314,color:#FFFFFF;
classDef cls_condition fill:#422006,color:#FFFFFF
classDef cls_control fill:#713f12,color:#FFFFFF
classDef cls_constraint fill:#854d0e,color:#FFFFFF;
classDef cls_system fill:#172554,color:#FFFFFF
classDef cls_application fill:#1e3a8a,color:#FFFFFF
classDef cls_component fill:#1e40af,color:#FFFFFF
classDef cls_interface fill:#082f49,color:#FFFFFF
classDef cls_artifact fill:#1e293b,color:#FFFFFF
classDef cls_asset fill:#334155,color:#FFFFFF;
classDef cls_data_store fill:#075985,color:#FFFFFF
classDef cls_test fill:#022c22,color:#FFFFFF;
classDef cls_deployment fill:#1e1b4b,color:#FFFFFF;
classDef cls_node fill:#312e81,color:#FFFFFF;
classDef cls_node_instance fill:#3730a3,color:#FFFFFF;
classDef cls_process fill:#2e1065,color:#FFFFFF
classDef cls_activity fill:#4c1d95,color:#FFFFFF
classDef cls_event fill:#5b21b6,color:#FFFFFF
classDef cls_state_machine fill:#4a044e,color:#FFFFFF
classDef cls_state fill:#701a75,color:#FFFFFF
classDef cls_risk fill:#881337,color:#FFFFFF;
classDef cls_threat fill:#4c0519,color:#FFFFFF;
classDef cls_note fill:#1f2937,color:#FFFFFF;

	class APP-001,APP-002,APP-003 cls_application;
	class ART-001,ART-002,ART-003,ART-004,ART-005,ART-006,ART-007,ART-008 cls_artifact;
	class COM-001,COM-002,COM-003,COM-004,COM-005,COM-006,COM-007,COM-008,COM-009,COM-010,COM-011,COM-012,COM-013,COM-014,COM-015 cls_component;
	class DTS-001,DTS-002 cls_data_store;
	class INT-001,INT-002,INT-003,INT-004,INT-005 cls_interface;
	class NOT-001 cls_note;
	class SYS-001 cls_system;
	class TES-001,TES-002,TES-003 cls_test;

```
