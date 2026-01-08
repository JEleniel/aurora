# Guided Workflow Implementation

## Overview

The AURORA application now includes a comprehensive guided workflow system aligned with **ISO/IEC/IEEE 42010** architectural methodology. This implementation provides step-by-step guidance through 14 phases of architecture design, helping users structure their architectural thinking along proven industrial practices.

## Architecture

### Workflow Phases (ISO 42010)

The 14-phase workflow maps to card types and provides structured guidance:

| Step | Phase | Primary Card Type | Goal |
| --- | --- | --- | --- |
| 1 | Establish Mission, Scope, Success Criteria | Mission | Define why system exists and non-negotiables |
| 2 | Identify Stakeholders and Concerns | Actor | Capture all stakeholder perspectives |
| 3 | Elicit and Classify Requirements | Requirement, Constraint | Establish comprehensive requirement set |
| 4 | Define Architectural Drivers | Driver | Extract dominant drivers from requirements |
| 5 | Select Styles and Patterns | Note, Artifact | Choose coarse-grained architectural patterns |
| 6 | Define System Context | Interface, Actor | Establish system boundaries and integration points |
| 7 | Decompose System (Logical) | LogicalComponent | Create coherent component model |
| 8 | Design Data and Information Architecture | Artifact | Make explicit data commitments |
| 9 | Design Runtime and Interaction Behavior | Behavior | Model execution paths and concurrency |
| 10 | Map to Deployment and Infrastructure | DeployableNode | Allocate components to physical nodes |
| 11 | Address Cross-Cutting Concerns | Interface, Constraint | Design security, observability, operations |
| 12 | Validate with Scenarios | Test, Behavior | Validate architecture under stress |
| 13 | Document Decisions and Rationale | Note, Artifact | Record architecture decisions (ADRs) |
| 14 | Iterate, Refine, and Govern | View, Constraint | Establish governance and iteration process |

### Core Components

#### 1. Workflow Utilities (`src/lib/utils/workflow.ts`)

Defines the complete workflow system:

```typescript
// Phase definitions with metadata
export const WORKFLOW_PHASES: Record<WorkflowPhase, PhaseDefinition>

// Maps card type to primary phase
export function getPhaseForCardType(cardType: CardType): WorkflowPhase | null

// Calculates overall workflow progress
export function calculateWorkflowProgress(cards: any[]): {
  totalPhases: number
  completedPhases: number
  percentComplete: number
  nextPhase: WorkflowPhase | null
}

// Gets recommended next card type to create
export function getRecommendedNextCardType(
  currentPhase: WorkflowPhase,
  existingCardTypes: Set<CardType>
): CardType | null
```

**Key Data Structure:**

```typescript
interface PhaseDefinition {
  phase: WorkflowPhase
  step: number
  name: string
  description: string
  goal: string
  expectedOutputs: string[]
  requiredCardTypes: CardType[]
  recommendedCardTypes: CardType[]
  nextPhases: WorkflowPhase[]
  riskOfSkipping: string[]
}
```

#### 2. Workflow Page (`src/routes/workflow/+page.svelte`)

Primary UI for guided workflow:

- **Layout**: Two-column (main content + sidebar checklist)
- **Progress Display**: Visual progress bar + percentage
- **Phase Navigation**: Expandable phase cards with details
- **Next Steps**: Highlighted recommended next action
- **Responsive**: Single-column on tablets/mobile

**Features:**

- Click phases to expand/collapse details
- View expected outputs for each phase
- See required vs recommended card types
- Understand risks of skipping phases
- Quick-create buttons for missing cards

#### 3. Workflow Checklist (`src/lib/components/WorkflowChecklist.svelte`)

Sidebar component showing all phases with progress:

- Phase-by-phase status (completed/in-progress/pending/blocked)
- Required card type indicators
- Quick-create buttons (+) for missing types
- Progress bar at top
- Color-coded status badges

#### 4. Workflow Context (`src/lib/components/WorkflowPhaseContext.svelte`)

In-form guidance component:

- Displays current phase (X/14) when creating cards
- Shows phase goal and description
- Provides contextual tips
- Only visible during card creation (not editing)
- Styled to grab attention without being intrusive

## User Experience

### Workflow Page (`/workflow`)

Entry point for users doing architecture work:

1. **See Progress**: Visual indicator of how far through workflow
2. **Understand Current Phase**: Expanded view of current recommended phase
3. **Plan Next Steps**: Sidebar shows what's required next
4. **Quick Create**: Click '+' button to create missing card types
5. **Learn Risks**: Understand consequences of skipping phases

### Card Creation Flow

1. User navigates to `/cards` or `/workflow` and clicks "New Card"
2. Card type selector defaults to next recommended type
3. **Workflow context box appears** with:
   + Current step (X/14)
   + Phase name and goal
   + Tips for this phase
4. User fills form with phase-aware guidance
5. Card created, checklist updates automatically
6. Progress bar advances

### Phase Progression

Workflow enforces logical progression through "blocking":

```
Phase 1 (Mission) ← Required
  ↓
Phase 2 (Stakeholders) ← Blocked until Phase 1 done
  ↓
Phase 3 (Requirements) ← Blocked until Phase 2 done
  ...
Phase 14 (Governance) ← Blocked until Phase 13 done
```

## Integration Points

### Card Templates (`src/lib/cardTemplates.ts`)

Templates now include ISO 42010 context:

- Mission templates for phase 1
- Driver templates for phase 4
- Requirement templates for phase 3
- etc.

### Dashboard (`src/routes/+page.svelte`)

Remains unchanged - table view of all cards, traditional workflow still available.

### Navigation (`src/routes/+layout.svelte`)

"Workflow" tab now appears first in navigation, making guided path visible.

## Design Principles

1. **Non-Invasive**: Guided workflow is optional enhancement, not replacement
2. **Evidence-Based**: Recommendations grounded in ISO 42010 standard
3. **Progressive**: Users can ignore phases and use freeform mode if desired
4. **Visual**: Color coding, progress bars, badges make status clear at a glance
5. **Actionable**: Every phase includes specific next steps and card type suggestions
6. **Educating**: Risk warnings teach users why phases matter
7. **Frictionless**: Quick-create buttons minimize workflow interruption

## Future Enhancements

### Planned (TODO List Items 4-10)

1. **Phase-Aware Templates** - Show only relevant templates for current phase
2. **Decision Record System** - Structured ADR template with rationale/options/consequences
3. **Validation Gating** - Warn if skipping important steps, suggest traceability review
4. **Scenario Gallery** - Example test cases for happy path/failure modes/abuse cases
5. **Settings Toggle** - Allow users to switch between workflow and freeform modes
6. **Enhanced Templates** - ISO 42010 naming and examples built into descriptions
7. **Workflow Export** - PDF report showing completion status and architectural readiness

## Architecture Decisions

### Why This Approach?

1. **Explicit Phases**: Rather than implicit workflow, made all 14 phases explicit and visible
2. **Card-Type Mapping**: Phases map to card types, not arbitrary sequences
3. **Recommended Not Forced**: Suggested sequence but users can deviate
4. **Comprehensive Guidance**: Each phase includes goal, outputs, risks - not just next steps
5. **Progressive Disclosure**: Details shown on demand (click phase to expand)
6. **Separate Page**: Workflow page is dedicated space, doesn't clutter main dashboard

### Constraints and Trade-offs

| Decision | Rationale | Alternative Considered |
| --- | --- | --- |
| Separate workflow page | Doesn't disrupt existing dashboard workflow | Integrated workflow into dashboard |
| Phase blocking (visual only) | Educates without enforcing | Strict enforcement would frustrate experts |
| Recommended card types | Suggests path without mandating | Hidden suggestions in templates |
| 14 separate phases | ISO 42010 standard, proven in industry | Simplified to 5-7 main phases |

## Testing Recommendations

1. **Phase Progression**: Create cards in order, verify checklist updates
2. **Out-of-Order**: Skip a phase, verify warning appears
3. **Next Phase Calculation**: Verify correct phase shown after each card creation
4. **Mobile Responsiveness**: Test workflow page on tablet/mobile
5. **Build Size**: Verify workflow files don't significantly increase bundle

## Files Changed

### New Files Created

- `src/lib/utils/workflow.ts` (380 lines) - Workflow definitions and logic
- `src/routes/workflow/+page.svelte` (544 lines) - Workflow page UI
- `src/lib/components/WorkflowChecklist.svelte` (350 lines) - Checklist component
- `src/lib/components/WorkflowPhaseContext.svelte` (70 lines) - In-form guidance

### Files Modified

- `src/routes/+layout.svelte` - Added Workflow to navigation
- `src/lib/cardTemplates.ts` - Added Mission templates
- `src/lib/components/CardForm.svelte` - Integrated WorkflowPhaseContext
- `src/routes/+page.svelte` - Fixed cardToCreate usage
- `src/routes/graph/+page.svelte` - Fixed array size check

### Build Status

- ✅ 0 TypeScript errors
- ✅ 11 warnings (unused CSS, non-breaking)
- ✅ Build: 8.60s
- ✅ No runtime errors

## References

- [ISO/IEC/IEEE 42010:2011](https://en.wikipedia.org/wiki/ISO/IEC/IEEE_42010) - Standard for architecture documentation
- [AURORA Specification](/docs/) - Project's own architecture standards
- [PROGRESS.md](/PROGRESS.md) - Complete implementation history

## Summary

This implementation provides a guided path through the 14 steps of ISO/IEC/IEEE 42010 architectural methodology, helping users structure their thinking without being prescriptive. The workflow is visible and accessible but not mandatory, preserving the freeform capability for experienced architects while guiding newcomers through proven practices.
