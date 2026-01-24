# MIS-001: Provide Default Tooling for AURORA - Deployment

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR


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
	DEP-001[\"`**Deployment**: DEP-001<br />Local Development`"/]
	DEP-002[\"`**Deployment**: DEP-002<br />User Environment`"/]
	DTS-001[("`**Data Store**: DTS-001<br />Workspace File System`")]
	DTS-002[("`**Data Store**: DTS-002<br />Extension State Store`")]
	NOD-001[/"`**Node**: NOD-001<br />User Workstation`"/]
	NOD-002[/"`**Node**: NOD-002<br />VSCode Extension Host`"/]
	NOD-003[/"`**Node**: NOD-003<br />Webview Sandbox`"/]
	NOD-004[/"`**Node**: NOD-004<br />Tauri Runtime`"/]


	COM-001 -- includes --> COM-009;
	COM-001 -- includes --> COM-010;
	COM-001 -- includes --> COM-011;
	COM-002 -- uses --> COM-003;
	COM-003 -- uses --> COM-001;
	COM-008 -- uses --> COM-001;
	COM-009 -- comprises --> COM-010;
	COM-010 -- comprises --> COM-011;
	COM-012 -- uses --> COM-001;
	COM-013 -- uses --> COM-008;
	COM-013 -- uses --> COM-015;
	COM-014 -- uses --> COM-013;
	COM-015 -- uses --> COM-001;
	COM-015 -- persists_to --> DTS-001;
	COM-015 -- persists_to --> DTS-002;
	DEP-001 -- includes --> NOD-001;
	DEP-002 -- includes --> NOD-001;
	DEP-002 -- includes --> NOD-002;
	DEP-002 -- includes --> NOD-003;
	DEP-002 -- includes --> NOD-004;
	NOD-001 -- hosts --> DTS-001;
	NOD-002 -- hosts --> COM-013;
	NOD-002 -- hosts --> DTS-002;
	NOD-003 -- hosts --> COM-014;
	NOD-004 -- hosts --> COM-003;


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

	class COM-001,COM-002,COM-003,COM-004,COM-005,COM-006,COM-007,COM-008,COM-009,COM-010,COM-011,COM-012,COM-013,COM-014,COM-015 cls_component;
	class DTS-001,DTS-002 cls_data_store;
	class DEP-001,DEP-002 cls_deployment;
	class NOD-001,NOD-002,NOD-003,NOD-004 cls_node;

```
