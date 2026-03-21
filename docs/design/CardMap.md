# Card Map

This reference visualizes the canonical card types and configured relationships from [`Aurora.modelconfiguration.json`](../../.github/aurora/reference/Aurora.modelconfiguration.json).

## Mermaid Graph

```mermaid
---
config:
  flowchart:
    defaultRenderer: "elk"
---
flowchart TB
	subgraph Structure
		APP["APP"]
		ART["ART"]
		COM["COM"]
		DST["DST"]
		INT["INT"]
		SYS["SYS"]
		subgraph External
			VND["VND"]
			API["API"]
			DSR["DSR"]
		end
		subgraph State
			EVT["EVT"]
			PRD["PRD"]
			STA["STA"]
			STM["STM"]
		end
		subgraph Deployment
			DEP["DEP"]
			NOD["NOD"]
			DAR["DAR"]
		end
	end

	subgraph Governance
		MIS["MIS"]
		DRI["DRI"]
		STK["STK"]
		subgraph Requirements
			EXC["EXC"]
			TES["TES"]
			ADR["ADR"]
			CTL["CTL"]
			CAP["CAP"]
			CNS["CNS"]
			FEA["FEA"]
			REQ["REQ"]
			STR["STR"]
		end
		subgraph Risk
			subgraph ThreatDiamond
				ADV["ADV"]
				AST["AST"]
				ROW["ROW"]
				THC["THC"]
				VIC["VIC"]
			end
			RIS["RIS"]
			THD["THD"]
			THM["THM"]
		end
		subgraph Process
			subgraph ProcessSteps
				ATV["ATV"]
				ACT["ACT"]
				CON["CON"]
				TRG["TRG"]
			end
			PRO["PRO"]
		end
	end

    MIS -->|"establishes"| DRI
	MIS -->|"has"| STK
    MIS -->|"necessitates"| SYS
	DRI -->|"drives"| REQ
	DRI -->|"imposes"| CNS
    STK -->|"define"| ADR
	STK -->|"accepts"| EXC
	EXC -->|"refines"| REQ
	ADR -->|"refines"| REQ
    REQ -->|"requires"| CAP
	REQ -->|"imposes"| CTL
	REQ -->|"defines"| TES
	CAP -->|"requires"| PRO
    CNS -->|"limits"| REQ
	CTL -->|"mitigates"| RIS
    FEA -->|"realizes"| CAP
    STK -->|"desires"| STR
    STR -->|"defines"| REQ
    TES -->|"verifies"| FEA

    PRO -->|"starts with"| ATV
    PRO -->|"involves"| ACT
    ACT -->|"performs"| ATV
    CON -->|"branches to"| ATV
    CON -->|"branches to"| CON
	CON -->|"branches to"| TRG
    ATV -->|"leads to"| ATV
    ATV -->|"triggers"| CON
	ATV -->|"causes" | TRG
    TRG -->|"triggers"| ATV
    TRG -->|"triggers"| CON
    TRG -->|"triggers"| TRG

    STM -->|"starts in"| STA
    STA -->|"triggers"| PRD
    STA -->|"triggers"| EVT
    STA -->|"transitions to"| STA
    EVT -->|"triggers"| EVT
    EVT -->|"triggers"| STA
    EVT -->|"triggers"| PRD
    PRD -->|"branches to"| PRD
    PRD -->|"branches to"| STA
    PRD -->|"branches to"| EVT

    DEP -->|"includes"| NOD
	NOD -->|"runs"| DAR

    SYS -->|"integrates"| APP
	SYS -->|"involves"| VND
    APP -->|"comprises"| COM
	APP -->|"generates"| DAR
    APP -->|"implements"| FEA
    APP -->|"implements"| TES
    ART -->|"persists to"| DST
	ART -->|"is"| AST
    COM -->|"calls"| INT
    COM -->|"composes"| COM
    COM -->|"executes"| STM
    COM -->|"exposes"| INT
    COM -->|"produces"| ART
	COM -->|"calls"| API
    DSR -->|"provides"| ART
	DSR -->|"provides"| DAR
	DST -->|"provides"| DAR
    VND -->|"provides"| API
    INT -->|"accepts"| ART
    INT -->|"returns"| ART

    THM -->|"includes"| THD
    THM -->|"defines"| RIS
	THD -->|"involves"| ADV
	ADV -->|"develops"| THC
	THC -->|"impacts"| VIC
	VIC -->|"is"| ROW
	ROW -->|"owns"| AST
	ADV -->|"exploits"| AST
	AST -->|"hosts"| THC
```

## Notes

- Nodes are shown by acronym, matching the canonical configuration.
- Edge labels are the configured relationship verbs.
- Card types without their own outgoing relationship list are still included as nodes so incoming links remain visible.
