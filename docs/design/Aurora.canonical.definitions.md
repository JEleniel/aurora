# Aurora Canonical Definitions

This document renders the full set of canonical card types and their allowed outgoing relationships as defined in [Aurora.canonical.definitions.json](../../.github/agents/aurora/Aurora.canonical.definitions.json).

## Graph

```mermaid
---
config:
  flowchart:
    defaultRenderer: "elk"
---
graph
    subgraph Requirements["Requirements & Intent"]
        MIS["Mission (MIS)"]
        DRI["Driver (DRI)"]
        STK["Stakeholder (STK)"]
        CAP["Capability (CAP)"]
        REQ["Requirement (REQ)"]
        FEA["Feature (FEA)"]
        ADR["ADR (ADR)"]
        CNS["Constraint (CNS)"]
        STR["Story (STR)"]
    end

    subgraph Structure["Structural"]
        SYS["System (SYS)"]
        APP["Application (APP)"]
        COM["Component (COM)"]
        INT["Interface (INT)"]
        TES["Test (TES)"]
    end

    subgraph Data["Data & Assets"]
        ART["Artifact (ART)"]
        AST["Asset (AST)"]
        DSR["Data Source (DSR)"]
        DST["Data Store (DST)"]
    end

    subgraph Deployment["Deployment"]
        DEP["Deployment (DEP)"]
        NOD["Node (NOD)"]
    end

    subgraph Process["Process & Activity"]
        ACT["Actor (ACT)"]
        PRO["Process (PRO)"]
        ATV["Activity (ATV)"]
        TRG["Trigger (TRG)"]
        CON["Condition (CON)"]
    end

    subgraph StateMachine["State Machine"]
        STM["State Machine (STM)"]
        STA["State (STA)"]
        PRD["Predicate (PRD)"]
        EVT["Event (EVT)"]
    end

    subgraph ThreatRisk["Threat & Risk"]
        THM["Threat Model (THM)"]
        THD["Threat Diamond (THD)"]
        ADV["Adversary (ADV)"]
        THC["Threat Capability (THC)"]
        VIC["Victim (VIC)"]
        ROW["Resource Owner (ROW)"]
        CTL["Control (CTL)"]
        RIS["Risk (RIS)"]
    end

    ATV -->|leads to| ATV
    ATV -->|receives| TRG
    ATV -->|triggers| CON
    ATV -->|uses| COM
    ATV -->|produces| ART

    ACT -->|performs| ATV

    STK -->|desires| STR

    ART -->|derives from| ART
    ART -->|is input into| COM
    ART -->|goes into| ATV
    ART -->|is| AST
    ART -->|persists to| DST

    CAP -->|requires| PRO

    SYS -->|integrates| APP

    APP -->|comprises| COM

    COM -->|composes| COM
    COM -->|implements| FEA
    COM -->|exposes| INT
    COM -->|implements| TES
    COM -->|produces| ART
    COM -->|includes| DSR
    COM -->|stores in| DST
    COM -->|executes| STM

    DST -->|retrieves| ART

    CON -->|branches to| ATV
    CON -->|branches to| TRG
    CON -->|branches to| CON

    CNS -->|limits| REQ

    CTL -->|mitigates| RIS
    CTL -->|protects| AST
    CTL -->|enforces| CNS
    CTL -->|governs| DST

    DEP -->|includes| NOD

    DSR -->|provides| ART

    DRI -->|drives| REQ

    EVT -->|transitions to| STA
    EVT -->|triggers| EVT
    EVT -->|triggers| PRD

    FEA -->|realizes| CAP
    FEA -->|implies| CNS

    MIS -->|establishes| DRI
    MIS -->|involves| STK
    MIS -->|necessitates| SYS
    MIS -->|necessitates| APP
    MIS -->|involves| DEP
    MIS -->|has| THM

    NOD -->|hosts| COM
    NOD -->|hosts| DST
    NOD -->|hosts| DSR

    PRD -->|branches to| STA
    PRD -->|branches to| PRD
    PRD -->|branches to| EVT

    PRO -->|includes| ATV
    PRO -->|includes| TRG
    PRO -->|branches on| CON
    PRO -->|involves| ACT

    REQ -->|requires| CAP
    REQ -->|has| ADR
    REQ -->|imposes| CNS
    REQ -->|requires| CTL

    STA -->|transitions to| STA
    STA -->|evaluates| PRD
    STA -->|handles| EVT

    STM -->|has| STA

    STR -->|explains| REQ
    STR -->|implies| CNS

    TES -->|verifies| FEA

    THD -->|threatens| AST
    THD -->|involves| ADV
    THD -->|uses| THC
    THD -->|impacts| VIC
    THD -->|creates| RIS

    ROW -->|owns| AST

    THM -->|includes| THD
    THM -->|includes| ROW

    TRG -->|initiates| ATV
    TRG -->|initiates| CON
    TRG -->|triggers| TRG
```
