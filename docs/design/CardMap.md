# Card Map

This reference visualizes the canonical card types and configured relationships from [`Aurora.modelconfiguration.json`](../../.github/aurora/reference/Aurora.modelconfiguration.json).

## Mermaid Graph

```mermaid
flowchart LR
    ATV["ATV"]
    ACT["ACT"]
    STK["STK"]
    ADR["ADR"]
    ART["ART"]
    AST["AST"]
    CAP["CAP"]
    SYS["SYS"]
    APP["APP"]
    COM["COM"]
    DST["DST"]
    CON["CON"]
    CNS["CNS"]
    CTL["CTL"]
    DEP["DEP"]
    DSR["DSR"]
    DRI["DRI"]
    EVT["EVT"]
    FEA["FEA"]
    INT["INT"]
    MIS["MIS"]
    NOD["NOD"]
    PRD["PRD"]
    PRO["PRO"]
    REQ["REQ"]
    RIS["RIS"]
    STA["STA"]
    STM["STM"]
    STR["STR"]
    TES["TES"]
    THD["THD"]
    ADV["ADV"]
    THC["THC"]
    VIC["VIC"]
    ROW["ROW"]
    THM["THM"]
    TRG["TRG"]
    VND["VND"]

    ATV -->|"leads to"| ATV
    ATV -->|"receives"| TRG
    ATV -->|"triggers"| CON
    ATV -->|"uses"| COM
    ATV -->|"produces"| ART

    ACT -->|"performs"| ATV

    STK -->|"desires"| STR

    ART -->|"derives from"| ART
    ART -->|"is input into"| COM
    ART -->|"goes into"| ATV
    ART -->|"is"| AST
    ART -->|"persists to"| DST

    CAP -->|"requires"| PRO

    SYS -->|"integrates"| APP

    APP -->|"comprises"| COM
    APP -->|"deploys to"| DEP

    COM -->|"composes"| COM
    COM -->|"implements"| FEA
    COM -->|"calls"| INT
    COM -->|"exposes"| INT
    COM -->|"implements"| TES
    COM -->|"produces"| ART
    COM -->|"includes"| DSR
    COM -->|"stores in"| DST
    COM -->|"executes"| STM
    COM -->|"runs on"| NOD

    DST -->|"retrieves"| ART
    DST -->|"runs on"| NOD

    CON -->|"branches to"| ATV
    CON -->|"branches to"| TRG
    CON -->|"branches to"| CON

    CNS -->|"limits"| REQ

    CTL -->|"mitigates"| RIS
    CTL -->|"protects"| AST
    CTL -->|"enforces"| CNS
    CTL -->|"governs"| DST
    CTL -->|"governs"| REQ
    CTL -->|"obstructs"| THC

    DEP -->|"includes"| NOD

    DSR -->|"provides"| ART
    DSR -->|"runs on"| NOD

    DRI -->|"drives"| REQ

    EVT -->|"transitions to"| STA
    EVT -->|"emits"| EVT
    EVT -->|"triggers"| PRD
    EVT -->|"carries"| ART

    FEA -->|"realizes"| CAP
    FEA -->|"implies"| CNS

    INT -->|"accepts"| ART
    INT -->|"returns"| ART

    MIS -->|"establishes"| DRI
    MIS -->|"involves"| STK
    MIS -->|"involves"| ACT
    MIS -->|"involves"| VND
    MIS -->|"necessitates"| SYS
    MIS -->|"necessitates"| APP
    MIS -->|"remediates"| THM

    PRD -->|"branches to"| STA
    PRD -->|"branches to"| PRD
    PRD -->|"emits"| EVT

    PRO -->|"includes"| ATV
    PRO -->|"includes"| TRG
    PRO -->|"branches on"| CON
    PRO -->|"involves"| ACT

    REQ -->|"requires"| CAP
    REQ -->|"has"| ADR
    REQ -->|"imposes"| CNS

    STA -->|"transitions to"| STA
    STA -->|"evaluates"| PRD
    STA -->|"handles"| EVT

    STM -->|"has"| STA

    STR -->|"explains"| REQ
    STR -->|"implies"| CNS

    TES -->|"verifies"| FEA

    THD -->|"threatens"| AST
    THD -->|"involves"| ADV
    THD -->|"uses"| THC
    THD -->|"impacts"| VIC
    THD -->|"creates"| RIS

    ROW -->|"owns"| AST

    THM -->|"includes"| THD
    THM -->|"includes"| ROW
    THM -->|"recommends"| CTL

    TRG -->|"initiates"| ATV
    TRG -->|"initiates"| CON
    TRG -->|"triggers"| TRG

    VND -->|"provides"| APP
    VND -->|"provides"| SYS
```

## Notes

- Nodes are shown by acronym, matching the canonical configuration.
- Edge labels are the configured relationship verbs.
- Card types without their own outgoing relationship list are still included as nodes so incoming links remain visible.
