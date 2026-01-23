# MIS-002: Enable Aurora Viewer And Editor - Deployment

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR


	COM-001[["`**Component**: COM-001<br />Model Home Scanner`"]]
	COM-002[["`**Component**: COM-002<br />Model Graph Indexer`"]]
	COM-003[["`**Component**: COM-003<br />Tree View Controller`"]]
	COM-004[["`**Component**: COM-004<br />Graph View Controller`"]]
	COM-005[["`**Component**: COM-005<br />View Filter Engine`"]]
	COM-006[["`**Component**: COM-006<br />Card Editor Controller`"]]
	COM-007[["`**Component**: COM-007<br />Card Markdown Renderer`"]]
	COM-008[["`**Component**: COM-008<br />Autosave And Undo Manager`"]]
	COM-009[["`**Component**: COM-009<br />Filesystem Watcher`"]]
	COM-010[["`**Component**: COM-010<br />Conflict Resolver`"]]
	COM-011[["`**Component**: COM-011<br />Model Validator`"]]
	COM-012[["`**Component**: COM-012<br />Audit And Hash Manager`"]]
	COM-013[["`**Component**: COM-013<br />Link Safety Engine`"]]
	COM-014[["`**Component**: COM-014<br />aurora_cli Invoker`"]]
	COM-015[["`**Component**: COM-015<br />Output Browser`"]]
	COM-016[["`**Component**: COM-016<br />SVG Exporter`"]]
	COM-017[["`**Component**: COM-017<br />MCP Server`"]]
	COM-018[["`**Component**: COM-018<br />Security Gate`"]]
	COM-019[["`**Component**: COM-019<br />Accessibility And Theme Manager`"]]
	COM-020[["`**Component**: COM-020<br />VS Code Host Adapter`"]]
	COM-021[["`**Component**: COM-021<br />IndraDB Store Adapter`"]]
	COM-022[["`**Component**: COM-022<br />Model Import/Export Manager`"]]
	DTS-001[("`**Data Store**: DTS-001<br />Aurora Model Home Folder (Import/Export)`")]
	DTS-002[("`**Data Store**: DTS-002<br />Derived Output Folder`")]
	DTS-003[("`**Data Store**: DTS-003<br />Undo History Store`")]
	DTS-004[("`**Data Store**: DTS-004<br />Local IndraDB Store`")]


	COM-001 -- uses --> DTS-001;
	COM-001 -- uses --> COM-018;
	COM-002 -- uses --> COM-011;
	COM-003 -- uses --> COM-001;
	COM-003 -- uses --> COM-018;
	COM-004 -- uses --> COM-002;
	COM-004 -- uses --> COM-005;
	COM-004 -- uses --> COM-019;
	COM-005 -- uses --> COM-002;
	COM-006 -- uses --> COM-011;
	COM-006 -- uses --> COM-013;
	COM-006 -- uses --> COM-008;
	COM-008 -- uses --> COM-012;
	COM-008 -- uses --> COM-018;
	COM-008 -- uses --> DTS-003;
	COM-009 -- uses --> COM-010;
	COM-009 -- uses --> DTS-001;
	COM-010 -- uses --> COM-011;
	COM-013 -- uses --> COM-011;
	COM-014 -- uses --> DTS-002;
	COM-015 -- uses --> DTS-002;
	COM-016 -- uses --> DTS-002;
	COM-017 -- uses --> COM-011;
	COM-017 -- uses --> COM-008;
	COM-018 -- uses --> DTS-001;
	COM-021 -- uses --> DTS-004;
	COM-021 -- uses --> COM-018;
	COM-022 -- uses --> DTS-001;
	COM-022 -- uses --> DTS-004;
	COM-022 -- uses --> COM-001;
	COM-022 -- uses --> COM-018;


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

	class COM-001,COM-002,COM-003,COM-004,COM-005,COM-006,COM-007,COM-008,COM-009,COM-010,COM-011,COM-012,COM-013,COM-014,COM-015,COM-016,COM-017,COM-018,COM-019,COM-020,COM-021,COM-022 cls_component;
	class DTS-001,DTS-002,DTS-003,DTS-004 cls_data_store;

```
