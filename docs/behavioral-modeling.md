# Behavioral Modeling in AURORA

This guide shows how to model behavioral aspects—sequences, state machines, workflows, and interactions—using AURORA cards and attributes.

---

## Overview

AURORA represents behavior through:

1. **`behavior` Cards**: Each discrete behavior or use case is its own card.
2. **`attributes`**: Structured data capturing state machines, sequences, and interactions.
3. **Links**: Relationships between behaviors, actors, interfaces, and requirements.

Unlike UML sequence diagrams or activity diagrams, AURORA's approach is **data-centric**: the structure of behavior is encoded in attributes, and tooling can generate visual representations.

---

## Use Cases and Actors

### Define Actors

Create `actor` cards for each user role or external system:

```json
{
  "id": "actor:end-user",
  "type": "actor",
  "name": "Use Application",
  "description": "Human end-users who interact with the system to accomplish tasks.",
  "version": "1.0.0",
  "status": "approved"
}
```

### Define Behaviors as Interactions

Create a `behavior` card for each significant use case or interaction:

```json
{
  "id": "behavior:user-authentication",
  "type": "behavior",
  "name": "Authenticate User",
  "description": "Users authenticate to the system to obtain access tokens and personalized sessions.",
  "version": "1.0.0",
  "status": "approved",
  "attributes": {
    "actors": ["actor:end-user"],
    "preconditions": [
      "User has valid credentials",
      "System is operational"
    ],
    "main_flow": [
      "User submits username and password",
      "System validates credentials against identity store",
      "System generates session token",
      "System returns token to user"
    ],
    "postconditions": [
      "User has active session",
      "Access token is valid for 1 hour"
    ],
    "alternate_flows": {
      "invalid_credentials": [
        "User submits incorrect password",
        "System logs failed attempt",
        "System returns 'Invalid credentials' error"
      ],
      "account_locked": [
        "User has exceeded login attempts",
        "System locks account temporarily",
        "System prompts for password reset"
      ]
    },
    "related_interfaces": ["interface:public-api"]
  },
  "links": ["link:behavior-auth-satisfies-req-security"]
}
```

**Key Attributes**:

- `actors`: Array of actor card IDs involved in the behavior.
- `preconditions`: Requirements that must be true before the behavior executes.
- `main_flow`: Ordered list of steps in the primary flow.
- `postconditions`: Guaranteed state after behavior completes.
- `alternate_flows`: Named alternative paths (error cases, branches).
- `related_interfaces`: Interfaces through which the behavior is exposed.

**Link to Requirements**:
Use a `verified-by` or `satisfies` link to connect the behavior to acceptance criteria or requirements.

---

## Sequence and Interaction Modeling

For multi-step interactions involving multiple components or systems, represent the sequence as an ordered list of messages:

```json
{
  "id": "behavior:api-data-retrieval",
  "type": "behavior",
  "name": "Retrieve Data via Public API",
  "description": "External client retrieves application data through the public API.",
  "attributes": {
    "actors": ["actor:external-system"],
    "interfaces": ["interface:public-api"],
    "sequence": [
      {
        "step": 1,
        "from": "external-system",
        "to": "api-gateway",
        "message": "HTTP GET /api/v1/data?filter=active",
        "protocol": "HTTP/REST"
      },
      {
        "step": 2,
        "from": "api-gateway",
        "to": "authentication-service",
        "message": "validate_token(token)",
        "protocol": "internal"
      },
      {
        "step": 3,
        "from": "authentication-service",
        "to": "api-gateway",
        "message": "token_valid(user_id)",
        "protocol": "internal"
      },
      {
        "step": 4,
        "from": "api-gateway",
        "to": "data-service",
        "message": "get_data(filter, user_id)",
        "protocol": "internal"
      },
      {
        "step": 5,
        "from": "data-service",
        "to": "api-gateway",
        "message": "[{id: 1, ...}, {id: 2, ...}]",
        "protocol": "internal"
      },
      {
        "step": 6,
        "from": "api-gateway",
        "to": "external-system",
        "message": "HTTP 200 [{id: 1, ...}, {id: 2, ...}]",
        "protocol": "HTTP/REST"
      }
    ],
    "constraints": ["constraint:latency", "constraint:compliance"]
  }
}
```

**Key Attributes**:

- `sequence`: Ordered array of message/interaction steps.
- Each step includes: `step` (number), `from`, `to`, `message`, `protocol`.
- `constraints`: References to non-functional requirements (latency, compliance).

---

## State Machine Modeling

Represent behavior with states and transitions:

```json
{
  "id": "behavior:order-fulfillment",
  "type": "behavior",
  "name": "Fulfill Customer Order",
  "description": "Process and fulfill a customer order through stages of preparation and delivery.",
  "attributes": {
    "state_machine": {
      "initial_state": "pending",
      "states": {
        "pending": {
          "description": "Order received; awaiting payment confirmation."
        },
        "paid": {
          "description": "Payment confirmed; ready for fulfillment."
        },
        "preparing": {
          "description": "Order being prepared for shipment."
        },
        "shipped": {
          "description": "Order in transit to customer."
        },
        "delivered": {
          "description": "Order delivered to customer."
        },
        "cancelled": {
          "description": "Order cancelled by customer or system."
        }
      },
      "transitions": [
        {
          "from": "pending",
          "to": "paid",
          "trigger": "payment_confirmed",
          "guard": "payment_valid == true",
          "action": "send_confirmation_email()"
        },
        {
          "from": "pending",
          "to": "cancelled",
          "trigger": "customer_cancels",
          "guard": "true",
          "action": "refund_payment(), notify_customer()"
        },
        {
          "from": "paid",
          "to": "preparing",
          "trigger": "begin_fulfillment",
          "guard": "inventory_available == true",
          "action": "reserve_inventory(), notify_warehouse()"
        },
        {
          "from": "preparing",
          "to": "shipped",
          "trigger": "shipment_picked_up",
          "guard": "tracking_number != null",
          "action": "send_tracking_email()"
        },
        {
          "from": "shipped",
          "to": "delivered",
          "trigger": "delivery_confirmed",
          "guard": "true",
          "action": "send_delivery_receipt(), update_inventory()"
        }
      ]
    },
    "related_interfaces": ["interface:public-api"],
    "actors": ["actor:customer", "actor:warehouse-operator"]
  }
}
```

**Key Attributes**:

- `state_machine`: Structured state machine definition.
    + `initial_state`: Starting state.
    + `states`: Dictionary of state names to state descriptions.
    + `transitions`: Array of state transitions.
        - `from`, `to`: State names.
        - `trigger`: Event or condition that fires the transition.
        - `guard`: Boolean condition that must be true.
        - `action`: Side effects (function calls, notifications).

---

## Activity/Process Flow Modeling

For workflows with parallel and conditional branches, use an activity structure:

```json
{
  "id": "behavior:deployment-pipeline",
  "type": "behavior",
  "name": "Deploy Release",
  "description": "Automated deployment pipeline from code commit to production.",
  "attributes": {
    "workflow": {
      "start": "code_committed",
      "activities": [
        {
          "id": "build",
          "name": "Build",
          "type": "action",
          "description": "Compile code and run unit tests.",
          "inputs": ["code_changes"],
          "outputs": ["build_artifact"]
        },
        {
          "id": "security_scan",
          "name": "Security Scan",
          "type": "action",
          "description": "Run static analysis and vulnerability checks.",
          "inputs": ["build_artifact"],
          "outputs": ["scan_report"]
        },
        {
          "id": "approve_manual",
          "name": "Manual Approval",
          "type": "decision",
          "description": "Human review and approval gate.",
          "inputs": ["scan_report"],
          "options": {
            "approved": "proceed to deploy",
            "rejected": "stop and notify"
          }
        },
        {
          "id": "deploy_staging",
          "name": "Deploy to Staging",
          "type": "action",
          "description": "Deploy to staging environment.",
          "inputs": ["build_artifact"],
          "outputs": ["staging_deployment"]
        },
        {
          "id": "run_smoke_tests",
          "name": "Run Smoke Tests",
          "type": "action",
          "description": "Execute smoke tests against staging.",
          "inputs": ["staging_deployment"],
          "outputs": ["test_results"]
        },
        {
          "id": "deploy_prod",
          "name": "Deploy to Production",
          "type": "action",
          "description": "Deploy to production environment.",
          "inputs": ["build_artifact"],
          "outputs": ["prod_deployment"]
        }
      ],
      "flows": [
        { "from": "code_committed", "to": "build" },
        { "from": "build", "to": "security_scan" },
        { "from": "security_scan", "to": "approve_manual" },
        { "from": "approve_manual", "to": "deploy_staging", "condition": "approved" },
        { "from": "approve_manual", "to": "end", "condition": "rejected" },
        { "from": "deploy_staging", "to": "run_smoke_tests" },
        { "from": "run_smoke_tests", "to": "deploy_prod", "condition": "all_pass" },
        { "from": "run_smoke_tests", "to": "end", "condition": "any_fail" },
        { "from": "deploy_prod", "to": "end" }
      ]
    },
    "automation": {
      "trigger": "on_commit_to_main",
      "timeout_minutes": 60,
      "notification_on_failure": true
    },
    "related_interfaces": ["interface:events-bus"]
  }
}
```

**Key Attributes**:

- `workflow`: Activity flow definition.
    + `start`: Initial activity or event.
    + `activities`: Array of steps (action, decision, synchronization).
    + `flows`: Directed edges between activities with optional conditions.
- `automation`: Metadata about execution (trigger, timeout, notifications).

---

## Event and Message Flow

For systems with event-driven or message-based architecture:

```json
{
  "id": "behavior:event-publication",
  "type": "behavior",
  "name": "Publish Events",
  "description": "Publish domain events to the event bus when important state changes occur.",
  "attributes": {
    "event_catalog": [
      {
        "event_name": "UserCreated",
        "description": "Emitted when a new user account is created.",
        "schema": {
          "type": "object",
          "properties": {
            "user_id": { "type": "string" },
            "email": { "type": "string" },
            "created_at": { "type": "string", "format": "date-time" }
          }
        },
        "publishers": ["behavior:user-registration"],
        "subscribers": ["behavior:send-welcome-email", "behavior:initialize-profile"]
      },
      {
        "event_name": "OrderPlaced",
        "description": "Emitted when a customer places an order.",
        "schema": {
          "type": "object",
          "properties": {
            "order_id": { "type": "string" },
            "customer_id": { "type": "string" },
            "total_amount": { "type": "number" },
            "timestamp": { "type": "string", "format": "date-time" }
          }
        },
        "publishers": ["behavior:checkout"],
        "subscribers": ["behavior:charge-payment", "behavior:fulfill-order", "behavior:notify-warehouse"]
      }
    ],
    "related_interfaces": ["interface:events-bus"]
  }
}
```

**Key Attributes**:

- `event_catalog`: Array of event definitions.
    + `event_name`, `description`, `schema` (JSON Schema for payload).
    + `publishers`, `subscribers`: Behavior cards that emit and consume the event.

---

## Linking Behaviors to Requirements

Use explicit links to trace behavior implementation to requirements:

```json
{
  "id": "link:behavior-auth-satisfies-req-auth",
  "type": "link",
  "source": "behavior:user-authentication",
  "target": "requirement:user-authentication",
  "link_type": "satisfies",
  "rationale": "This behavior implements the user authentication requirement by providing a complete flow from credential submission to token issuance.",
  "strength": "strong"
}
```

Or use inline relations for simpler associations:

```json
{
  "id": "behavior:user-authentication",
  ...
  "relations": ["requirement:user-authentication", "actor:end-user", "interface:public-api"]
}
```

---

## Constraints on Behavior

Reference constraint cards to capture non-functional requirements:

```json
{
  "id": "behavior:api-data-retrieval",
  ...
  "constraints": [
    "constraint:latency",  // Must complete within SLA
    "constraint:compliance" // Must comply with data residency rules
  ]
}
```

---

## Best Practices

1. **One Behavior = One Card**: Each distinct use case, process, or significant interaction gets its own behavior card.
2. **Link to Actors and Interfaces**: Always indicate which actors and interfaces are involved.
3. **Provide Acceptance Criteria**: Add `acceptance_criteria` field to make behavior testable.
4. **Document Preconditions and Postconditions**: Essential for understanding when behavior is valid.
5. **Use Structures, Not Prose**: Model sequences, states, and flows as structured attributes, not free-form text.
6. **Reference Constraints**: Link to constraint cards for non-functional requirements.
7. **Create Test Cards**: For each behavior, create corresponding `test` cards to verify acceptance criteria.

---

## Generating Diagrams from Behavior Cards

Tooling can generate:

- **Sequence Diagrams**: From `sequence` attributes.
- **State Machines**: From `state_machine` attributes.
- **Activity Diagrams**: From `workflow` attributes.
- **Use Case Diagrams**: From aggregation of `behavior` cards and their `actors`.
- **Event Flow Diagrams**: From `event_catalog` attributes.

Example: Tooling reads the `behavior:user-authentication` card and renders a sequence diagram showing interactions between user, API gateway, authentication service, and downstream systems.

---

## See Also

- [Card Field Reference](card-field-reference.md)
- [Link Types](link-types.md)
- [Constraint Modeling Guide](constraint-modeling.md)
- [Tooling Guide](tooling.md)
