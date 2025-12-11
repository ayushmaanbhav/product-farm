---
layout: default
title: Why Product-FARM
---

# Why Product-FARM?

## The Problem We Solve

Every software product has business logic. Pricing calculations, eligibility rules, risk assessments, approval workflows—these rules define how your product behaves. But how are these rules typically managed?

### The Current Reality

<div class="problem-grid">

**Scattered Across Code**
```java
// Hidden in service layer
if (customer.age > 65 && customer.income < 50000) {
    premium = basePremium * 1.5;
} else if (customer.riskScore > 7) {
    premium = basePremium * 1.3;
}
// ... 500 more lines scattered across 20 files
```

**Buried in Spreadsheets**
```
Excel files passed between teams
- v1_final.xlsx
- v1_final_FINAL.xlsx
- v1_final_FINAL_approved_v2.xlsx
No version control. No audit trail.
```

**Lost in Configuration Files**
```yaml
# config/pricing_rules.yaml
# Last modified: ???
# By whom: ???
# Why: ???
rules:
  - condition: "age > 65"
    action: "multiply 1.5"
```

</div>

### The Consequences

| Problem | Impact |
|---------|--------|
| **Fragmented Logic** | Business rules scattered across codebases, spreadsheets, and documents |
| **No Single Source of Truth** | Different systems calculate differently, causing inconsistencies |
| **Change is Risky** | Modifying rules requires code deployments; one mistake affects production |
| **No Visibility** | Business teams can't see or understand the actual rules running |
| **No Audit Trail** | When rules change, there's no record of who changed what and why |
| **Slow Time-to-Market** | Every rule change requires developer involvement and release cycles |

---

## The Solution: Product-FARM

Product-FARM is a **centralized rule engine platform** that transforms how you manage business logic.

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   BEFORE: Rules scattered everywhere                            │
│                                                                 │
│   [Code] ←→ [Spreadsheets] ←→ [Config] ←→ [Docs]               │
│      ↑           ↑              ↑           ↑                   │
│      └───────────┴──────────────┴───────────┘                   │
│                    (Chaos)                                      │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   AFTER: Product-FARM as the single source of truth            │
│                                                                 │
│                  ┌─────────────────┐                            │
│                  │  Product-FARM   │                            │
│                  │  Rule Engine    │                            │
│                  └────────┬────────┘                            │
│                           │                                     │
│         ┌─────────────────┼─────────────────┐                   │
│         ↓                 ↓                 ↓                   │
│   [Microservices]   [Web Apps]      [Mobile Apps]              │
│                                                                 │
│              (Consistency & Control)                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core Benefits

### 1. Visual Rule Management

Create and modify business rules through an intuitive visual interface—no coding required.

![Rule Builder](screenshots/rule-builder-expression.png)

- **Drag-and-drop** rule construction
- **Visual DAG** showing rule dependencies
- **Real-time validation** to catch errors before deployment

### 2. Single Source of Truth

All business rules live in one place. Every system, every team, every calculation uses the same logic.

```
Product Definition
├── Datatypes (currency, percentage, age)
├── Attributes (customer_age, coverage_amount, risk_level)
├── Rules (calculate_premium, assess_risk, apply_discount)
└── Functionalities (quote, underwrite, renew)
```

### 3. Complete Audit Trail

Every change is tracked. Know who changed what, when, and why.

```json
{
  "change_id": "chg_892734",
  "timestamp": "2024-01-15T14:32:00Z",
  "user": "jane.smith@company.com",
  "action": "UPDATE_RULE",
  "rule_name": "calculate_premium",
  "previous_value": "base * 1.2",
  "new_value": "base * 1.25",
  "reason": "Q1 pricing adjustment per JIRA-4521"
}
```

### 4. Lightning-Fast Execution

Built in Rust with a tiered compilation engine for maximum performance.

| Tier | Technology | Latency | Use Case |
|------|------------|---------|----------|
| **Tier 0** | AST Interpretation | ~1.15µs | Initial evaluations |
| **Tier 1** | Bytecode VM | ~330ns | Hot paths (3.5x faster) |

### 5. Parallel Rule Execution

Rules are organized in a **Directed Acyclic Graph (DAG)** and executed in parallel where possible.

![DAG Visualization](screenshots/rules-dag-full-view.png)

```
Level 0 (Parallel):     Level 1:           Level 2:
├── base_premium        │                  │
├── age_factor    ────► final_premium ───► monthly_payment
├── smoker_factor       │
└── risk_level          │
```

### 6. AI-Powered Assistance

Built-in AI assistant helps create rules, debug issues, and optimize performance.

```
User: "Create a rule that gives 10% discount for customers
       who have been with us more than 5 years"

AI: I'll create a loyalty discount rule for you:

    Name: apply_loyalty_discount
    Expression: IF years_as_customer > 5 THEN price * 0.9 ELSE price
    Inputs: [years_as_customer, price]
    Outputs: [discounted_price]
```

---

## Comparison: Product-FARM vs. Alternatives

| Aspect | Hardcoded Rules | Spreadsheets | Generic Rule Engines | **Product-FARM** |
|--------|-----------------|--------------|---------------------|------------------|
| **Visual Interface** | ❌ | ❌ | Partial | ✅ Full visual builder |
| **Version Control** | Via git (code only) | ❌ | Partial | ✅ Built-in |
| **Audit Trail** | ❌ | ❌ | Partial | ✅ Complete |
| **Performance** | Varies | N/A | Moderate | ✅ Sub-microsecond |
| **Business User Access** | ❌ | ✅ | Partial | ✅ Full access |
| **DAG Execution** | ❌ | ❌ | ❌ | ✅ Automatic parallel |
| **AI Assistance** | ❌ | ❌ | ❌ | ✅ Built-in |
| **Type Safety** | Language-dependent | ❌ | Partial | ✅ Full type system |

---

## Who Is Product-FARM For?

### Product Managers
- Define business logic without writing code
- Visualize how rules interact and affect outcomes
- Test scenarios before going live
- Track changes and understand their impact

### Business Analysts
- Translate business requirements into executable rules
- Validate rules against expected outcomes
- Document rule logic in a structured format
- Collaborate with technical teams effectively

### Developers
- Focus on building features, not maintaining rule spaghetti
- Integrate via REST or gRPC APIs
- Trust that business logic is correct and consistent
- Deploy rule changes without code releases

### Compliance & Audit Teams
- Complete audit trail of all rule changes
- Understand exactly how decisions are made
- Verify regulatory compliance
- Generate reports on rule behavior

---

## Real-World Impact

### Before Product-FARM

```
Time to change a pricing rule: 2-3 weeks
- Write ticket (1 day)
- Developer picks up (3-5 days)
- Code, test, review (3-5 days)
- Deploy to staging (1-2 days)
- UAT (3-5 days)
- Production deploy (1-2 days)

Risk of errors: HIGH
Visibility: LOW
```

### After Product-FARM

```
Time to change a pricing rule: 2-3 hours
- Update rule in visual editor (30 min)
- Test with simulation (30 min)
- Review and approve (1 hour)
- Automatic deployment (instant)

Risk of errors: LOW (type-checked, validated)
Visibility: COMPLETE (full audit trail)
```

---

## The Vision

Product-FARM isn't just a rule engine—it's the foundation for a **product-centric architecture** where:

1. **Products are First-Class Citizens**: Every business capability is modeled as a product with clear inputs, outputs, and rules

2. **Microservices Consume Products**: Services don't implement business logic—they consume it from Product-FARM

3. **Consistency Across Channels**: Web, mobile, API, batch—all use the same rules

4. **Business and Tech Aligned**: Both teams work from the same source of truth

```
                    ┌─────────────────────┐
                    │   Product-FARM      │
                    │   (Source of Truth) │
                    └──────────┬──────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
        ▼                      ▼                      ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│ Pricing       │    │ Underwriting  │    │ Claims        │
│ Service       │    │ Service       │    │ Service       │
└───────────────┘    └───────────────┘    └───────────────┘
        │                      │                      │
        └──────────────────────┼──────────────────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        ▼                      ▼                      ▼
   [Web App]            [Mobile App]           [Partner API]
```

---

## Get Started

Ready to transform how you manage business logic?

<div class="cta-buttons">

[Quick Start Guide](QUICK_START) - Get running in 5 minutes

[Core Concepts](CONCEPTS) - Understand the fundamentals

[Architecture](ARCHITECTURE) - Technical deep-dive

</div>

---

## Summary

Product-FARM exists because **business logic deserves better than scattered code and spreadsheets**.

We believe that:
- Rules should be **visible** to everyone who needs them
- Changes should be **tracked** and **auditable**
- Business users should be able to **participate** in rule management
- Performance should never be **compromised** for usability
- Systems should have a **single source of truth**

That's why we built Product-FARM.
