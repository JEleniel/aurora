# Constraint Modeling in AURORA

This guide explains how to model constraints—non-functional requirements, performance bounds, and compliance rules—in AURORA.

---

## Overview

Constraints in AURORA are represented by:

1. **`constraint` Cards**: Each discrete constraint is its own first-class element.
2. **`attributes`**: Structured data capturing bounds, units, compliance rules, and formulas.
3. **Links**: Relationships to the requirements, components, or behaviors they govern.

Constraints are essential for capturing non-functional requirements (NFR) that define _how well_ the system must perform, not _what_ it does.

---

## Types of Constraints

### Performance Constraints

Capture timing, throughput, and resource usage requirements:

```json
{
  "id": "constraint:latency",
  "type": "constraint",
  "name": "Limit Request Latency",
  "description": "All API requests must respond within acceptable latency bounds to provide a responsive user experience.",
  "version": "1.0.0",
  "status": "approved",
  "attributes": {
    "category": "performance",
    "metric": "response_time_ms",
    "bounds": {
      "p50": 100,
      "p95": 500,
      "p99": 1000,
      "p99_9": 2000
    },
    "unit": "milliseconds",
    "applies_to": ["behavior:api-data-retrieval", "behavior:user-authentication"],
    "measurement_method": "synthetic monitoring; endpoint /health/latency",
    "enforcement": "SLO-driven alerting; page on breach"
  },
  "audit_history": [
    { "event": "created", "by": "performance-team", "event_time": "2025-12-09T10:00:00Z" },
    { "event": "approved", "by": "architecture-review", "event_time": "2025-12-10T14:30:00Z" }
  ]
}
```

**Key Attributes**:

- `category`: Type of constraint (performance, compliance, capacity, etc.).
- `metric`: Name of the measured attribute (e.g., response_time_ms, throughput_rps).
- `bounds`: Acceptable range; may be quantile-based (p50, p95, p99) or min/max.
- `unit`: Measurement unit (ms, rps, MB, etc.).
- `applies_to`: Array of behavior/component card IDs affected by this constraint.
- `measurement_method`: How the constraint is measured or validated.
- `enforcement`: Action taken when constraint is violated.

### Throughput Constraints

```json
{
  "id": "constraint:throughput",
  "type": "constraint",
  "name": "Ensure Sufficient Throughput",
  "description": "System must handle peak load without degradation.",
  "attributes": {
    "category": "performance",
    "metric": "requests_per_second",
    "bounds": {
      "minimum": 1000,
      "peak": 5000
    },
    "unit": "rps",
    "applies_to": ["logical-component:api-gateway"],
    "test_method": "load testing with k6 or JMeter",
    "failure_mode": "queue requests; alert on queue depth > 10000"
  }
}
```

### Storage and Capacity Constraints

```json
{
  "id": "constraint:storage",
  "type": "constraint",
  "name": "Limit Storage Usage",
  "description": "Constrain total storage to manage costs and performance.",
  "attributes": {
    "category": "capacity",
    "metric": "total_storage_gb",
    "bounds": {
      "soft_limit": 500,
      "hard_limit": 1000
    },
    "unit": "GB",
    "applies_to": ["deployable-node:primary-database"],
    "growth_rate": "~50GB per month",
    "archival_policy": "archive data older than 2 years",
    "enforcement": "alert at 80% capacity; fail writes at 100%"
  }
}
```

---

## Security and Compliance Constraints

### Data Protection

```json
{
  "id": "constraint:data-protection",
  "type": "constraint",
  "name": "Protect Sensitive Data",
  "description": "All sensitive data must be encrypted at rest and in transit.",
  "version": "1.0.0",
  "status": "approved",
  "attributes": {
    "category": "security",
    "applies_to": ["logical-component:data-store", "interface:public-api"],
    "rules": [
      {
        "rule": "encryption_at_rest",
        "requirement": "AES-256 encryption for all persistent storage",
        "verification": "code review; infrastructure audit"
      },
      {
        "rule": "encryption_in_transit",
        "requirement": "TLS 1.2+ for all network communication",
        "verification": "SSL/TLS configuration audit"
      },
      {
        "rule": "key_management",
        "requirement": "Keys stored in FIPS 140-2 certified HSM",
        "verification": "access logs; key rotation audit"
      }
    ],
    "standards": ["PCI-DSS", "HIPAA", "GDPR"],
    "enforcement": "quarterly security audit; penetration testing"
  }
}
```

**Key Attributes**:

- `rules`: Array of specific security rules with verification methods.
- `standards`: Compliance frameworks (PCI-DSS, HIPAA, SOC 2, GDPR, etc.).
- `enforcement`: Audit and testing approach.

### Audit and Access Control

```json
{
  "id": "constraint:audit-logging",
  "type": "constraint",
  "name": "Provide Auditability and Access Controls",
  "description": "All user actions and privileged operations must be logged and reviewable.",
  "attributes": {
    "category": "compliance",
    "applies_to": ["interface:admin-console", "behavior:user-authentication"],
    "audit_requirements": {
      "events_logged": [
        "user login/logout",
        "data access",
        "configuration changes",
        "permission modifications"
      ],
      "retention_period_days": 2555,
      "log_immutability": "logs cannot be modified or deleted once written",
      "integrity_check": "cryptographic checksums on audit records"
    },
    "access_control": {
      "model": "role-based access control (RBAC)",
      "roles": ["admin", "operator", "auditor", "user"],
      "principle": "least privilege; deny by default"
    },
    "enforcement": "monthly access review; quarterly audit trail verification"
  }
}
```

---

## Structural Constraints

### Deployment and Resource Constraints

```json
{
  "id": "constraint:availability",
  "type": "constraint",
  "name": "Ensure High Availability",
  "description": "System must maintain 99.95% uptime (maximum 2.16 hours downtime per month).",
  "attributes": {
    "category": "reliability",
    "metric": "uptime_percentage",
    "bounds": {
      "target": 99.95,
      "minimum_acceptable": 99.9
    },
    "applies_to": ["logical-component:api-gateway", "deployable-node:primary-database"],
    "strategies": [
      "multi-region deployment",
      "automatic failover",
      "health checks every 30 seconds"
    ],
    "sla_definition": {
      "availability": "99.95%",
      "rto_minutes": 5,
      "rpo_minutes": 15
    },
    "enforcement": "continuous monitoring; alert on availability < 99.9%; incident review on breaches"
  }
}
```

**Key Attributes**:

- `sla_definition`: Recovery Time Objective (RTO) and Recovery Point Objective (RPO).
- `strategies`: Implementation approaches (redundancy, failover, etc.).

### Scalability Constraints

```json
{
  "id": "constraint:scalability",
  "type": "constraint",
  "name": "Support Elastic Scaling",
  "description": "System must scale horizontally to handle load variations without service interruption.",
  "attributes": {
    "category": "scalability",
    "applies_to": ["logical-component:microservices"],
    "scaling_policies": {
      "horizontal": {
        "min_instances": 2,
        "max_instances": 100,
        "scale_up_trigger": "cpu > 70% for 2 minutes",
        "scale_down_trigger": "cpu < 20% for 10 minutes"
      }
    },
    "testing": "load testing to 10x peak load; chaos engineering tests",
    "enforcement": "automated scaling; capacity planning reviews quarterly"
  }
}
```

---

## Interoperability Constraints

### API and Format Constraints

```json
{
  "id": "constraint:api-compatibility",
  "type": "constraint",
  "name": "Maintain API Stability",
  "description": "Public API must maintain backward compatibility; breaking changes require major version bump and 12-month deprecation period.",
  "attributes": {
    "category": "interoperability",
    "applies_to": ["interface:public-api"],
    "versioning_policy": "semantic versioning (major.minor.patch)",
    "deprecation_timeline": {
      "announce": "at version X",
      "support_period_months": 12,
      "removal": "at version X+1"
    },
    "compatibility_rules": [
      "no required parameter additions without new API version",
      "no response field removals",
      "new fields may be added; default to null or sensible default"
    ],
    "enforcement": "API contract testing; breaking change detection in CI"
  }
}
```

---

## Compliance Constraints

### Regulatory Constraints

```json
{
  "id": "constraint:compliance",
  "type": "constraint",
  "name": "Enforce Data Residency",
  "description": "Customer data must be stored in the customer's region of residence per GDPR and local data protection regulations.",
  "version": "1.0.0",
  "status": "approved",
  "attributes": {
    "category": "compliance",
    "applies_to": ["logical-component:data-store", "deployable-node:primary-database"],
    "regulations": ["GDPR", "CCPA", "local data protection laws"],
    "rules": [
      {
        "rule": "data_residency",
        "requirement": "EU customer data must reside in EU region; US customer data in US region",
        "verification": "database location audit; backup location verification"
      },
      {
        "rule": "data_transfer",
        "requirement": "Cross-border transfers require standard contractual clauses and adequacy determinations",
        "verification": "legal review; data transfer agreements audit"
      }
    ],
    "enforcement": "quarterly compliance audit; legal review of any changes"
  }
}
```

---

## Linking Constraints to Elements

### In Requirements

Link constraints to requirements that must satisfy them:

```json
{
  "id": "requirement:api-performance",
  "type": "requirement",
  "name": "Provide Low-Latency API",
  "description": "API endpoints must respond quickly to support real-time user interactions.",
  "constraints": ["constraint:latency", "constraint:throughput"],
  "acceptance_criteria": [
    "p95 latency < 500ms",
    "p99 latency < 1000ms",
    "throughput >= 1000 rps"
  ]
}
```

### In Behaviors

Reference constraints in behavior definitions:

```json
{
  "id": "behavior:api-data-retrieval",
  "type": "behavior",
  ...
  "constraints": ["constraint:latency", "constraint:data-protection"]
}
```

### In Components

Reference constraints on logical and physical components:

```json
{
  "id": "logical-component:api-gateway",
  "type": "logical-component",
  ...
  "constraints": ["constraint:availability", "constraint:throughput", "constraint:data-protection"]
}
```

---

## Creating Links Between Constraints and Other Elements

Use explicit links to document the relationship:

```json
{
  "id": "link:constraint-latency-governs-behavior-api",
  "type": "link",
  "source": "constraint:latency",
  "target": "behavior:api-data-retrieval",
  "link_type": "governs",  // Not a standard link type; may be custom
  "rationale": "This behavior must comply with the latency constraint to provide acceptable user experience.",
  "strength": "strong"
}
```

Alternatively, use inline relations:

```json
{
  "id": "constraint:latency",
  ...
  "relations": ["behavior:api-data-retrieval", "logical-component:api-gateway", "requirement:api-performance"]
}
```

---

## Constraint Hierarchies

Group related constraints into a constraint domain or category:

```json
{
  "id": "constraint-group:security",
  "type": "note",  // Or a custom "constraint-group" type
  "name": "Security Constraints",
  "description": "All constraints related to data protection, access control, and compliance.",
  "relations": [
    "constraint:data-protection",
    "constraint:audit-logging",
    "constraint:api-authentication"
  ]
}
```

---

## Validating and Testing Constraints

### Test Cards

Create test cards that verify constraints are met:

```json
{
  "id": "test:latency-measurement",
  "type": "test",
  "name": "Measure API Latency",
  "description": "Synthetic monitoring test to measure API response latencies.",
  "version": "1.0.0",
  "status": "verified",
  "attributes": {
    "test_type": "performance",
    "framework": "k6",
    "test_script": "tests/latency.js",
    "metrics": ["p50", "p95", "p99", "p99_9"],
    "frequency": "every 5 minutes"
  },
  "links": ["link:test-verifies-constraint-latency"]
}
```

### Acceptance Criteria

Constraints should be verifiable via acceptance criteria:

```json
{
  "id": "requirement:api-performance",
  ...
  "acceptance_criteria": [
    "p95 latency measured over 24 hours is < 500ms",
    "throughput sustained at 5000 rps without queue overflow",
    "no P1/P2 performance regressions between releases"
  ]
}
```

---

## Best Practices

1. **Quantify Constraints**: Use measurable metrics; avoid vague language like "fast" or "secure."
2. **Specify Units**: Always include units (ms, rps, GB, %, etc.).
3. **Define Measurement Methods**: How is the constraint verified? Monitoring? Load testing? Audit?
4. **Provide Enforcement Mechanism**: What happens when the constraint is violated?
5. **Link Broadly**: Connect constraints to all affected requirements, behaviors, and components.
6. **Establish Baselines**: Record current performance as a baseline for tracking improvement or regression.
7. **Review Regularly**: Constraints should be reviewed with stakeholders as the system evolves.
8. **Document Exceptions**: If a constraint cannot be met in specific cases, document the exception and rationale.

---

## See Also

- [Card Field Reference](card-field-reference.md)
- [Behavioral Modeling Guide](behavioral-modeling.md)
- [Link Types](link-types.md)
- [Tooling Guide](tooling.md)
