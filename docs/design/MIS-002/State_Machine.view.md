# MIS-002: Enable Aurora Viewer And Editor - State Machine

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR


	CON-001{{"`**Condition**: CON-001<br />Schema And Invariants Valid`"}}
	CON-002{{"`**Condition**: CON-002<br />External Change Conflicts With Local Edit`"}}
	CON-003{{"`**Condition**: CON-003<br />Filesystem Write Is Safe`"}}
	EVT-001["`**Event**: EVT-001<br />Model Home Selected`"]@{shape: tri}
	EVT-002["`**Event**: EVT-002<br />Filesystem Change Detected`"]@{shape: tri}
	EVT-003["`**Event**: EVT-003<br />Edit Initiated`"]@{shape: tri}
	EVT-004["`**Event**: EVT-004<br />Save Requested`"]@{shape: tri}
	EVT-005["`**Event**: EVT-005<br />Validation Passed`"]@{shape: tri}
	EVT-006["`**Event**: EVT-006<br />Validation Failed`"]@{shape: tri}
	EVT-007["`**Event**: EVT-007<br />Output Generation Or Export Requested`"]@{shape: tri}
	EVT-008["`**Event**: EVT-008<br />MCP Request Received`"]@{shape: tri}
	STA-001["`**State**: STA-001<br />Idle`"]@{shape: win-pane}
	STA-002["`**State**: STA-002<br />Model Loaded`"]@{shape: win-pane}
	STA-003["`**State**: STA-003<br />Editing`"]@{shape: win-pane}
	STA-004["`**State**: STA-004<br />Validating`"]@{shape: win-pane}
	STA-005["`**State**: STA-005<br />Resolving Conflict`"]@{shape: win-pane}
	STA-006["`**State**: STA-006<br />Generating Outputs`"]@{shape: win-pane}
	STA-007["`**State**: STA-007<br />Error`"]@{shape: win-pane}
	STM-001[\"`**State Machine**: STM-001<br />Editing Session Lifecycle`"\]


	EVT-002 -- triggers --> CON-002;
	EVT-004 -- triggers --> CON-001;
	EVT-004 -- triggers --> CON-003;
	STA-001 -- receives --> EVT-001;
	STA-001 -- transitions_to --> STA-002;
	STA-002 -- receives --> EVT-003;
	STA-002 -- receives --> EVT-002;
	STA-002 -- transitions_to --> STA-003;
	STA-002 -- transitions_to --> STA-005;
	STA-003 -- receives --> EVT-004;
	STA-003 -- receives --> EVT-002;
	STA-003 -- transitions_to --> STA-004;
	STA-003 -- transitions_to --> STA-005;
	STA-004 -- receives --> EVT-005;
	STA-004 -- receives --> EVT-006;
	STA-004 -- triggers --> CON-001;
	STA-004 -- transitions_to --> STA-006;
	STA-004 -- transitions_to --> STA-007;
	STA-005 -- receives --> EVT-002;
	STA-005 -- triggers --> CON-002;
	STA-005 -- transitions_to --> STA-002;
	STA-005 -- transitions_to --> STA-007;
	STA-006 -- receives --> EVT-007;
	STA-006 -- transitions_to --> STA-002;
	STA-007 -- receives --> EVT-006;
	STA-007 -- transitions_to --> STA-003;
	STA-007 -- transitions_to --> STA-001;
	STM-001 -- starts_in --> STA-001;
	STM-001 -- includes --> STA-001;
	STM-001 -- includes --> STA-002;
	STM-001 -- includes --> STA-003;
	STM-001 -- includes --> STA-004;
	STM-001 -- includes --> STA-005;
	STM-001 -- includes --> STA-006;
	STM-001 -- includes --> STA-007;
	STM-001 -- includes --> EVT-001;
	STM-001 -- includes --> EVT-002;
	STM-001 -- includes --> EVT-003;
	STM-001 -- includes --> EVT-004;
	STM-001 -- includes --> EVT-005;
	STM-001 -- includes --> EVT-006;
	STM-001 -- includes --> EVT-007;
	STM-001 -- includes --> EVT-008;
	STM-001 -- includes --> CON-001;
	STM-001 -- includes --> CON-002;
	STM-001 -- includes --> CON-003;


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

	class CON-001,CON-002,CON-003 cls_condition;
	class EVT-001,EVT-002,EVT-003,EVT-004,EVT-005,EVT-006,EVT-007,EVT-008 cls_event;
	class STA-001,STA-002,STA-003,STA-004,STA-005,STA-006,STA-007 cls_state;
	class STM-001 cls_state_machine;

```

