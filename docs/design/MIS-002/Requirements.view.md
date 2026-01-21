# MIS-002: Enable Aurora Viewer And Editor - Requirements

```mermaid
%%{init: {'flowchart': {'defaultRenderer': 'elk'}, 'themeVariables': { 'clusterBkg': 'transparent' }}}%%
graph LR


	ACT-001{{"`**Actor**: ACT-001<br />Architect`"}}
	ACT-002{{"`**Actor**: ACT-002<br />Tool User`"}}
	ACT-003{{"`**Actor**: ACT-003<br />Agent`"}}
	ACT-004{{"`**Actor**: ACT-004<br />Threat Actor`"}}
	CAP-001(["`**Capability**: CAP-001<br />Load And Index Model Home`"])
	CAP-002(["`**Capability**: CAP-002<br />Browse Model Tree`"])
	CAP-003(["`**Capability**: CAP-003<br />Navigate Model Graph`"])
	CAP-004(["`**Capability**: CAP-004<br />Filter Views`"])
	CAP-005(["`**Capability**: CAP-005<br />Edit Cards Safely`"])
	CAP-006(["`**Capability**: CAP-006<br />Persist Model Changes`"])
	CAP-007(["`**Capability**: CAP-007<br />Detect And Resolve Concurrent Changes`"])
	CAP-008(["`**Capability**: CAP-008<br />Validate Schema And Invariants`"])
	CAP-009(["`**Capability**: CAP-009<br />Maintain Audit Trail And Hash`"])
	CAP-010(["`**Capability**: CAP-010<br />Generate And Browse Derived Outputs`"])
	CAP-011(["`**Capability**: CAP-011<br />Export Official Diagrams To SVG`"])
	CAP-012(["`**Capability**: CAP-012<br />Expose MCP CRUD And Events`"])
	CAP-013(["`**Capability**: CAP-013<br />Accessible And Polished UI`"])
	CNS-001(["`**Constraint**: CNS-001<br />Cross-Platform Desktop Application`"])
	CNS-002(["`**Constraint**: CNS-002<br />WCAG AA Conformance`"])
	CNS-003(["`**Constraint**: CNS-003<br />Treat Model Content As Untrusted`"])
	CNS-004(["`**Constraint**: CNS-004<br />No tmp Directory Inside Model Homes`"])
	CNS-005(["`**Constraint**: CNS-005<br />Prefer aurora_cli For Derived Outputs`"])
	DRI-001(["`**Driver**: DRI-001<br />Graph-Centric Model Navigation`"])
	DRI-002(["`**Driver**: DRI-002<br />Safe Invariant-Preserving Editing`"])
	DRI-003(["`**Driver**: DRI-003<br />Canonical Output Parity`"])
	DRI-004(["`**Driver**: DRI-004<br />Live Modeling With Agents`"])
	DRI-005(["`**Driver**: DRI-005<br />Accessible Cross-Platform UX`"])
	FEA-001(["`**Feature**: FEA-001<br />Multi-Model Loader`"])
	FEA-002(["`**Feature**: FEA-002<br />Model Tree View`"])
	FEA-003(["`**Feature**: FEA-003<br />Graph Navigation View`"])
	FEA-004(["`**Feature**: FEA-004<br />View Filter Selector`"])
	FEA-005(["`**Feature**: FEA-005<br />Card JSON Editor`"])
	FEA-006(["`**Feature**: FEA-006<br />Rendered Markdown Preview`"])
	FEA-007(["`**Feature**: FEA-007<br />Autosave And Undo`"])
	FEA-008(["`**Feature**: FEA-008<br />File Watching And Conflict Resolution`"])
	FEA-009(["`**Feature**: FEA-009<br />Validation On Load Edit And Save`"])
	FEA-010(["`**Feature**: FEA-010<br />Audit Trail And Hash Updates`"])
	FEA-011(["`**Feature**: FEA-011<br />Safe Link Editor`"])
	FEA-012(["`**Feature**: FEA-012<br />Output Synchronization Via aurora_cli`"])
	FEA-013(["`**Feature**: FEA-013<br />View Browser`"])
	FEA-014(["`**Feature**: FEA-014<br />SVG Export`"])
	FEA-015(["`**Feature**: FEA-015<br />MCP Server And Change Events`"])
	FEA-016(["`**Feature**: FEA-016<br />Secure File Handling`"])
	FEA-017(["`**Feature**: FEA-017<br />Accessible And Styled UI`"])
	FEA-018(["`**Feature**: FEA-018<br />VS Code Extension UI`"])
	REQ-001(["`**Requirement**: REQ-001<br />Load Models From A Model Home`"])
	REQ-002(["`**Requirement**: REQ-002<br />Default To Lowest Mission`"])
	REQ-003(["`**Requirement**: REQ-003<br />Model Tabs Remember Context`"])
	REQ-004(["`**Requirement**: REQ-004<br />Model Tree Mirrors On-Disk Layout`"])
	REQ-005(["`**Requirement**: REQ-005<br />Tree Shows Only Card JSON Files`"])
	REQ-006(["`**Requirement**: REQ-006<br />Tree Never Shows tmp`"])
	REQ-007(["`**Requirement**: REQ-007<br />Graph View Displays Local Topology`"])
	REQ-008(["`**Requirement**: REQ-008<br />Click To Recenter`"])
	REQ-009(["`**Requirement**: REQ-009<br />View Filters Match aurora_cli`"])
	REQ-010(["`**Requirement**: REQ-010<br />Filtering Affects Visibility Only`"])
	REQ-011(["`**Requirement**: REQ-011<br />Edit Allowed Fields`"])
	REQ-012(["`**Requirement**: REQ-012<br />Save-As-You-Go With Conflict Handling`"])
	REQ-013(["`**Requirement**: REQ-013<br />Keep Derived Outputs In Sync`"])
	REQ-014(["`**Requirement**: REQ-014<br />Browse Views And Generated Markdown`"])
	REQ-015(["`**Requirement**: REQ-015<br />Export SVG Of Official Diagrams`"])
	REQ-016(["`**Requirement**: REQ-016<br />Watch Model Folders`"])
	REQ-017(["`**Requirement**: REQ-017<br />First Writer Wins For Conflicts`"])
	REQ-018(["`**Requirement**: REQ-018<br />Expose MCP Endpoint While Running`"])
	REQ-019(["`**Requirement**: REQ-019<br />MCP CRUD And Change Events`"])
	REQ-020(["`**Requirement**: REQ-020<br />Keyboard Navigation Works End-To-End`"])
	REQ-021(["`**Requirement**: REQ-021<br />Respect Reduced Motion Preferences`"])
	REQ-022(["`**Requirement**: REQ-022<br />Graph View Supports Zoom`"])
	REQ-023(["`**Requirement**: REQ-023<br />Editor Pane Has JSON And Markdown`"])
	REQ-024(["`**Requirement**: REQ-024<br />Undo Queue Is Available`"])
	REQ-025(["`**Requirement**: REQ-025<br />IDs Are Immutable`"])
	REQ-026(["`**Requirement**: REQ-026<br />Validate On Load Edit And Save`"])
	REQ-027(["`**Requirement**: REQ-027<br />Audit Trail And Version Bump`"])
	REQ-028(["`**Requirement**: REQ-028<br />Compute Hash On Save`"])
	REQ-029(["`**Requirement**: REQ-029<br />Link Creation Is Invariant Safe`"])
	REQ-030(["`**Requirement**: REQ-030<br />Paths And Filenames Are Safe`"])
	REQ-031(["`**Requirement**: REQ-031<br />Never Execute Model Content`"])
	REQ-032(["`**Requirement**: REQ-032<br />Retro-Futuristic Control-Panel Aesthetic`"])
	REQ-034(["`**Requirement**: REQ-034<br />VS Code Extension Reuses Core Logic`"])
	STR-001["`**Story**: STR-001<br />Edit Cards Safely With Feedback`"]@{shape: card}
	STR-002["`**Story**: STR-002<br />Export Official Views To SVG`"]@{shape: card}
	STR-003["`**Story**: STR-003<br />Agent Updates Model Via MCP`"]@{shape: card}


	ACT-001 -- desires --> STR-001;
	ACT-002 -- desires --> STR-002;
	ACT-003 -- desires --> STR-003;
	CAP-001 -- satisfies --> REQ-001;
	CAP-001 -- satisfies --> REQ-002;
	CAP-001 -- satisfies --> REQ-003;
	CAP-001 -- satisfies --> REQ-004;
	CAP-001 -- satisfies --> REQ-005;
	CAP-001 -- satisfies --> REQ-006;
	CAP-002 -- satisfies --> REQ-004;
	CAP-002 -- satisfies --> REQ-005;
	CAP-002 -- satisfies --> REQ-006;
	CAP-003 -- satisfies --> REQ-007;
	CAP-003 -- satisfies --> REQ-008;
	CAP-003 -- satisfies --> REQ-022;
	CAP-004 -- satisfies --> REQ-009;
	CAP-004 -- satisfies --> REQ-010;
	CAP-005 -- satisfies --> REQ-011;
	CAP-005 -- satisfies --> REQ-023;
	CAP-005 -- satisfies --> REQ-025;
	CAP-005 -- satisfies --> REQ-029;
	CAP-006 -- satisfies --> REQ-012;
	CAP-006 -- satisfies --> REQ-024;
	CAP-007 -- satisfies --> REQ-016;
	CAP-007 -- satisfies --> REQ-017;
	CAP-008 -- satisfies --> REQ-026;
	CAP-008 -- satisfies --> REQ-030;
	CAP-008 -- satisfies --> REQ-031;
	CAP-009 -- satisfies --> REQ-027;
	CAP-009 -- satisfies --> REQ-028;
	CAP-010 -- satisfies --> REQ-013;
	CAP-010 -- satisfies --> REQ-014;
	CAP-011 -- satisfies --> REQ-015;
	CAP-012 -- satisfies --> REQ-018;
	CAP-012 -- satisfies --> REQ-019;
	CAP-013 -- satisfies --> REQ-020;
	CAP-013 -- satisfies --> REQ-021;
	CAP-013 -- satisfies --> REQ-032;
	CNS-002 -- limits --> FEA-017;
	CNS-003 -- limits --> FEA-016;
	CNS-004 -- limits --> FEA-002;
	CNS-005 -- limits --> FEA-012;
	CNS-005 -- limits --> FEA-014;
	DRI-001 -- drives --> REQ-001;
	DRI-001 -- drives --> REQ-002;
	DRI-001 -- drives --> REQ-003;
	DRI-001 -- drives --> REQ-004;
	DRI-001 -- drives --> REQ-005;
	DRI-001 -- drives --> REQ-006;
	DRI-001 -- drives --> REQ-007;
	DRI-001 -- drives --> REQ-008;
	DRI-001 -- drives --> REQ-022;
	DRI-002 -- drives --> REQ-009;
	DRI-002 -- drives --> REQ-010;
	DRI-002 -- drives --> REQ-011;
	DRI-002 -- drives --> REQ-012;
	DRI-002 -- drives --> REQ-023;
	DRI-002 -- drives --> REQ-024;
	DRI-002 -- drives --> REQ-025;
	DRI-002 -- drives --> REQ-026;
	DRI-002 -- drives --> REQ-027;
	DRI-002 -- drives --> REQ-028;
	DRI-002 -- drives --> REQ-029;
	DRI-002 -- drives --> REQ-030;
	DRI-002 -- drives --> REQ-031;
	DRI-003 -- drives --> REQ-013;
	DRI-003 -- drives --> REQ-014;
	DRI-003 -- drives --> REQ-015;
	DRI-004 -- drives --> REQ-016;
	DRI-004 -- drives --> REQ-017;
	DRI-004 -- drives --> REQ-018;
	DRI-004 -- drives --> REQ-019;
	DRI-005 -- drives --> REQ-020;
	DRI-005 -- drives --> REQ-021;
	DRI-005 -- drives --> REQ-032;
	FEA-001 -- enables --> CAP-001;
	FEA-001 -- enables --> CAP-008;
	FEA-001 -- satisfies --> REQ-001;
	FEA-001 -- satisfies --> REQ-002;
	FEA-001 -- satisfies --> REQ-003;
	FEA-002 -- enables --> CAP-002;
	FEA-002 -- satisfies --> REQ-004;
	FEA-002 -- satisfies --> REQ-005;
	FEA-002 -- satisfies --> REQ-006;
	FEA-003 -- enables --> CAP-003;
	FEA-003 -- satisfies --> REQ-007;
	FEA-003 -- satisfies --> REQ-008;
	FEA-003 -- satisfies --> REQ-022;
	FEA-004 -- enables --> CAP-004;
	FEA-004 -- satisfies --> REQ-009;
	FEA-004 -- satisfies --> REQ-010;
	FEA-005 -- enables --> CAP-005;
	FEA-005 -- satisfies --> REQ-011;
	FEA-005 -- satisfies --> REQ-023;
	FEA-006 -- enables --> CAP-005;
	FEA-006 -- satisfies --> REQ-023;
	FEA-007 -- enables --> CAP-006;
	FEA-007 -- satisfies --> REQ-012;
	FEA-007 -- satisfies --> REQ-024;
	FEA-008 -- enables --> CAP-007;
	FEA-008 -- satisfies --> REQ-016;
	FEA-008 -- satisfies --> REQ-017;
	FEA-009 -- enables --> CAP-008;
	FEA-009 -- satisfies --> REQ-026;
	FEA-010 -- enables --> CAP-009;
	FEA-010 -- satisfies --> REQ-027;
	FEA-010 -- satisfies --> REQ-028;
	FEA-011 -- enables --> CAP-005;
	FEA-011 -- satisfies --> REQ-029;
	FEA-012 -- enables --> CAP-010;
	FEA-012 -- satisfies --> REQ-013;
	FEA-013 -- enables --> CAP-010;
	FEA-013 -- satisfies --> REQ-014;
	FEA-014 -- enables --> CAP-011;
	FEA-014 -- satisfies --> REQ-015;
	FEA-015 -- enables --> CAP-012;
	FEA-015 -- satisfies --> REQ-018;
	FEA-015 -- satisfies --> REQ-019;
	FEA-016 -- enables --> CAP-008;
	FEA-016 -- satisfies --> REQ-030;
	FEA-016 -- satisfies --> REQ-031;
	FEA-017 -- enables --> CAP-013;
	FEA-017 -- satisfies --> REQ-020;
	FEA-017 -- satisfies --> REQ-021;
	FEA-017 -- satisfies --> REQ-032;
	FEA-018 -- satisfies --> REQ-034;
	STR-001 -- explains --> CAP-005;
	STR-001 -- explains --> CAP-008;
	STR-001 -- explains --> FEA-009;
	STR-001 -- implies --> CNS-003;
	STR-002 -- explains --> CAP-011;
	STR-002 -- explains --> FEA-014;
	STR-003 -- explains --> CAP-012;
	STR-003 -- explains --> FEA-015;
	STR-003 -- implies --> CNS-003;


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

	class ACT-001,ACT-002,ACT-003,ACT-004 cls_actor;
	class CAP-001,CAP-002,CAP-003,CAP-004,CAP-005,CAP-006,CAP-007,CAP-008,CAP-009,CAP-010,CAP-011,CAP-012,CAP-013 cls_capability;
	class CNS-001,CNS-002,CNS-003,CNS-004,CNS-005 cls_constraint;
	class DRI-001,DRI-002,DRI-003,DRI-004,DRI-005 cls_driver;
	class FEA-001,FEA-002,FEA-003,FEA-004,FEA-005,FEA-006,FEA-007,FEA-008,FEA-009,FEA-010,FEA-011,FEA-012,FEA-013,FEA-014,FEA-015,FEA-016,FEA-017,FEA-018 cls_feature;
	class REQ-001,REQ-002,REQ-003,REQ-004,REQ-005,REQ-006,REQ-007,REQ-008,REQ-009,REQ-010,REQ-011,REQ-012,REQ-013,REQ-014,REQ-015,REQ-016,REQ-017,REQ-018,REQ-019,REQ-020,REQ-021,REQ-022,REQ-023,REQ-024,REQ-025,REQ-026,REQ-027,REQ-028,REQ-029,REQ-030,REQ-031,REQ-032,REQ-034 cls_requirement;
	class STR-001,STR-002,STR-003 cls_story;

```

