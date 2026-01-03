# Six-Layer Documentation Architecture

> **Status:** Draft
> **Last Updated:** 2025-12-31
> **Purpose:** Define granular boundaries, attributes, and functions for each documentation layer

---

## Overview

The Banoo platform documentation is organized into 6 distinct layers, each with specific:
- **Scope**: What this layer can know and express
- **Language**: Terminology and abstraction level
- **Attributes**: Data it can define
- **Functions**: Behaviors/operations it can describe
- **Boundaries**: What it explicitly CANNOT contain

```
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 6: MASTER SCHEMA                           │
│         (Ultimate Source of Truth - All Attributes & Relations)     │
├─────────────────────────────────────────────────────────────────────┤
│  LAYER 3: Backend/Tech    │  LAYER 4: Session UI  │  LAYER 5: Portal│
│  (Events, State, Logic)   │  (Instruments, Patch) │  (Reports, Mgmt)│
├───────────────────────────┴───────────────────────┴─────────────────┤
│                    LAYER 2: PRODUCT DOMAIN                          │
│              (Domain Language, Abstract from Tech)                  │
├─────────────────────────────────────────────────────────────────────┤
│                    LAYER 1: CLIENT REQUIREMENTS                     │
│              (Natural Language, Business Goals)                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Layer 1: Client Requirements

### Purpose
Captures what clients (companies using Banoo) want to assess and why. Written in natural language for non-technical stakeholders.

### Scope
- Business objectives for assessment
- Competencies to measure (in business terms)
- Scenario themes and contexts
- Success criteria (in outcome terms)
- Constraints and preferences

### Language Constraints
| Allowed | NOT Allowed |
|---------|-------------|
| "Leadership qualities" | "competency_scores array" |
| "Crisis handling ability" | "event_type: crisis_start" |
| "Communication effectiveness" | "WebSocket message payload" |
| "Decision-making under pressure" | "state mutation trigger" |
| "Technical problem-solving" | "JSON Patch operation" |

### Attributes (Read-Only from Layer 2)
```yaml
ClientRequirement:
  id: string                    # Reference ID
  client_name: string           # Company name
  assessment_goals: string[]    # What they want to learn
  target_roles: string[]        # "Senior Engineer", "Tech Lead"
  scenario_preferences:
    domains: string[]           # "infrastructure", "security", "data"
    duration_minutes: number    # Preferred length
    difficulty: string          # "standard", "challenging"
  competency_priorities:        # Ranked list
    - name: string              # "Crisis Management"
      weight: high|medium|low
  constraints:
    time_zone: string
    language: string
    accessibility_needs: string[]
```

### Functions
**NONE** - This layer only captures requirements, no logic.

### Boundaries
- Cannot reference technical implementation
- Cannot specify HOW to measure, only WHAT
- Cannot define scoring algorithms
- Cannot reference UI components or data structures

---

## Layer 2: Product/Domain Language

### Purpose
Translates client requirements into domain concepts. Uses assessment and game design terminology, abstracted from technical implementation.

### Scope
- Competency definitions and observables
- Scenario narratives and structures
- Agent personas and behaviors
- Evidence categories and importance
- Assessment flow and phases

### Language Constraints
| Allowed | NOT Allowed |
|---------|-------------|
| "Competency: Crisis Management" | "competency_id: uuid" |
| "Observable: Escalation timing" | "event.timestamp - session.start" |
| "Evidence strength: strong" | "evidence_score >= 0.8" |
| "Agent persona: Senior Engineer" | "agent.llm_config.temperature" |
| "Scenario phase: Detection" | "phase_index: 0" |
| "Candidate action: Acknowledge alert" | "action_type: monitoring/acknowledge" |

### Attributes
```yaml
Competency:
  name: string                      # "Crisis Management"
  description: string               # What this competency means
  observables:                      # What we look for
    - name: string                  # "Escalation Timing"
      description: string           # "How quickly candidate escalates"
      evidence_indicators:          # Behaviors that demonstrate this
        - positive: string          # "Escalates within 5 minutes"
        - negative: string          # "Never escalates despite severity"
      importance: critical|high|medium|low

Scenario:
  name: string                      # "Database Outage Crisis"
  domain: string                    # "infrastructure"
  narrative: string                 # Story description
  duration_minutes: number
  difficulty: standard|challenging|expert
  phases:
    - name: string                  # "Detection"
      description: string
      duration_range: [min, max]    # Minutes
      key_observables: string[]     # Which observables matter here

  backbone_events:                  # Events that always happen
    - name: string                  # "Initial Alert Fires"
      timing: string                # "start+2m" or "after:detection_complete"
      description: string

  branch_triggers:                  # Conditional events
    - name: string                  # "VP Escalation Path"
      condition: string             # Natural language condition
      events: string[]              # Event names from pool

AgentPersona:
  name: string                      # "Alex Chen"
  role: string                      # "Senior Backend Engineer"
  personality_traits: string[]      # ["detail-oriented", "cautious"]
  communication_style: string       # "technical, thorough"
  stress_response: string           # "becomes more terse"
  expertise_areas: string[]         # ["databases", "performance"]

  response_patterns:
    greeting: string                # How they say hello
    under_pressure: string          # How style changes
    when_uncertain: string          # How they express doubt

EvidenceCategory:
  name: string                      # "communication"
  description: string
  importance_levels:
    critical: string                # When to use critical
    high: string
    medium: string
    low: string
```

### Functions
```yaml
# Domain-level operations (no implementation)
Functions:
  - name: "Evaluate Observable"
    input: "Candidate action sequence"
    output: "Evidence strength for observable"
    description: "Determines how strongly an action sequence demonstrates an observable"

  - name: "Select Next Event"
    input: "Current scenario state, candidate actions"
    output: "Next event from backbone or branch"
    description: "Decides what happens next based on scenario rules"

  - name: "Generate Agent Response"
    input: "Conversation context, agent persona, scenario state"
    output: "Agent message"
    description: "Creates contextually appropriate agent response"
```

### Boundaries
- Cannot specify database schemas or API endpoints
- Cannot define UI component structures
- Cannot reference JSON Patch operations
- Cannot include scoring formulas (only evidence strength concepts)
- Cannot specify WebSocket message formats

---

## Layer 3: Backend/Tech Domain

### Purpose
Defines all server-side technical implementation: state structures, events, actions, validations, computed attributes, and business logic.

### Scope
- Session state schema (complete)
- Event definitions with payloads
- Action handlers and validations
- Evidence extraction rules
- Computed attributes and metrics
- State transition rules
- Scenario script execution logic

### Language Constraints
| Allowed | NOT Allowed |
|---------|-------------|
| `SessionState`, `Event`, `Action` | "Component", "Render", "CSS" |
| `event_type: string` | "onClick handler" |
| `payload: { alert_id: string }` | "display: flex" |
| `validation: min_length(10)` | "Tab component" |
| `computed: error_rate()` | "Modal visibility" |

### Attributes
```yaml
# === SESSION STATE ===
SessionState:
  session_id: uuid
  scenario_id: uuid
  candidate_id: uuid
  status: initializing|active|paused|completed|abandoned

  # Timing
  started_at: timestamp
  scenario_time: duration         # Elapsed scenario time
  real_time: duration             # Actual elapsed time
  time_limit: duration

  # Scenario execution
  current_phase: string
  backbone_position: number       # Index in backbone
  active_branches: string[]       # Currently active branch IDs
  triggered_events: string[]      # Events that have fired

  # Instrument states (detailed in Layer 4 references)
  chat: ChatState
  email: EmailState
  monitoring: MonitoringState
  task_board: TaskBoardState
  git_repository: GitState

# === EVENTS ===
Event:
  event_id: uuid
  session_id: uuid
  event_type: string              # Namespaced: "chat/message_received"
  timestamp: timestamp
  scenario_time: duration
  source: candidate|agent|system|timeline

  payload: object                 # Event-specific data

  evidence:
    category: communication|technical|decision|navigation
    importance: critical|high|medium|low
    tags: string[]
    observable_hints: string[]    # Which observables this might indicate

# === ACTIONS ===
Action:
  action_type: string             # "chat/send_message"
  payload: object

  # Validation rules
  validations:
    - field: string
      rule: required|min_length|max_length|pattern|enum|range
      params: object
      error_message: string

  # Evidence generation
  evidence_rules:
    category: string
    importance_formula: string    # Expression to compute importance
    tags: string[]
    extract_observables:          # What to extract for scoring
      - observable: string
        extraction: string        # How to extract value

# === COMPUTED ATTRIBUTES ===
ComputedAttribute:
  name: string                    # "error_rate"
  depends_on: string[]            # State paths this reads
  formula: string                 # Computation expression
  cache_duration: duration|none   # How long to cache

# Examples:
ComputedAttributes:
  - name: response_time_avg
    depends_on: ["events[].timestamp"]
    formula: "avg(events.filter(e => e.source == 'candidate').map(e => e.response_latency))"

  - name: escalation_score
    depends_on: ["events[].event_type", "scenario_time"]
    formula: |
      escalation_events = events.filter(e => e.tags.includes('escalation'))
      if escalation_events.empty: return 0
      first_escalation = escalation_events[0].scenario_time
      return normalize(first_escalation, expected_escalation_time)

# === EVIDENCE EXTRACTION ===
EvidenceRule:
  observable: string              # "escalation_timing"
  trigger_events: string[]        # Events that might produce evidence
  extraction:
    type: temporal|behavioral|content|pattern
    params: object
  strength_calculation: string    # Formula for evidence strength

# Examples:
EvidenceRules:
  - observable: "escalation_timing"
    trigger_events: ["chat/send_message", "email/send"]
    extraction:
      type: temporal
      params:
        look_for: "message to VP or manager"
        relative_to: "first_critical_alert"
    strength_calculation: |
      time_to_escalate = event.scenario_time - first_critical_alert.scenario_time
      if time_to_escalate < 5min: return "strong_positive"
      if time_to_escalate < 15min: return "moderate_positive"
      if time_to_escalate < 30min: return "weak"
      return "negative"

# === STATE TRANSITIONS ===
StateTransition:
  trigger: string                 # Event type or condition
  condition: string               # Optional additional condition
  mutations:                      # State changes to apply
    - path: string                # JSON path
      operation: set|append|remove|increment
      value: any|expression
```

### Functions
```yaml
Functions:
  # Event Processing
  - name: process_action
    signature: "(session_id: uuid, action: Action) -> Result<Event[], Error>"
    description: "Validates action, generates events, updates state"

  - name: apply_event
    signature: "(session: SessionState, event: Event) -> SessionState"
    description: "Applies event to session state, returns new state"

  # Scenario Execution
  - name: evaluate_backbone
    signature: "(session: SessionState) -> Event[]"
    description: "Check if any backbone events should fire"

  - name: evaluate_branches
    signature: "(session: SessionState, event: Event) -> Branch[]"
    description: "Check if event triggers any branches"

  - name: execute_branch
    signature: "(session: SessionState, branch: Branch) -> Event[]"
    description: "Execute branch and generate resulting events"

  # Evidence & Scoring
  - name: extract_evidence
    signature: "(event: Event, rules: EvidenceRule[]) -> Evidence[]"
    description: "Extract evidence from event using rules"

  - name: compute_observable_score
    signature: "(observable: string, evidence: Evidence[]) -> Score"
    description: "Compute score for observable from evidence"
```

### Boundaries
- Cannot define UI layout or styling
- Cannot specify component hierarchies
- Cannot reference React/frontend frameworks
- Cannot define JSON Patch format (that's Layer 4)
- Cannot include display text or labels (that's Layer 4)

---

## Layer 4: Session UI (Candidate Experience)

### Purpose
Defines the candidate-facing assessment interface: instruments, components, interactions, and real-time state synchronization via JSON Patch.

### Scope
- Instrument configurations and layouts
- Component registry and rendering
- JSON Patch message format (maps only, no arrays)
- UI state (local, not persisted)
- Interaction handlers (click, type, submit)
- Display formatting and labels

### Language Constraints
| Allowed | NOT Allowed |
|---------|-------------|
| "Component", "Instrument", "Tab" | "Evidence extraction" |
| "onClick", "onChange" | "Scoring formula" |
| "JSON Patch", "path" | "Database schema" |
| "visible", "enabled", "label" | "Validation rule" (use Layer 3) |
| "layout", "region", "grid" | "Agent LLM config" |

### Attributes
```yaml
# === INSTRUMENT CONFIGURATION ===
InstrumentConfig:
  type: chat|email|monitoring|task_board|git_repository
  id: string

  metadata:
    title: string                 # Display title
    icon: string                  # Icon identifier
    description: string
    accent_color: string

  behavior:
    readonly: boolean
    notifications: boolean
    auto_refresh: boolean

  layout:
    position: sidebar|main|modal|fullscreen
    default_size: { width: number, height: number }
    regions: map<string, RegionConfig>

  components: ComponentConfig[]   # Root components

# === COMPONENT CONFIGURATION ===
ComponentConfig:
  type: ComponentType
  id: string

  # Visibility
  visible: boolean|expression     # Expression evaluated against state
  enabled: boolean|expression

  # Data binding (references Layer 3 state paths)
  data_source: string             # "monitoring.alerts"
  data_transform: string          # Transform function name

  # Styling
  class_name: string
  variant: string
  size: sm|md|lg

  # Children
  children: ComponentConfig[]
  item_template: ComponentConfig  # For list rendering

  # Props (component-specific)
  props: map<string, any>

  # Events (emit actions to Layer 3)
  on_click: ActionConfig
  on_change: ActionConfig
  on_submit: ActionConfig

ComponentType:
  # Layout
  - tabs, tab_panel, accordion, split_pane
  - card_grid, list, tree, container, flex

  # Data Display
  - stat_card, metric_chart, data_table, timeline
  - status_badge, avatar, progress_bar, text

  # Interactive
  - button, button_group, dropdown, search_input
  - text_input, textarea, select, checkbox, toggle

  # Composite
  - modal, drawer, popover, tooltip, alert_banner

  # Domain-Specific
  - message_bubble, email_preview, task_card
  - pr_card, alert_card, service_card, commit_item

# === ACTION CONFIGURATION (UI -> Backend) ===
ActionConfig:
  action_type: string             # Maps to Layer 3 Action
  payload_template: map<string, expression>

  # Optional UI feedback
  optimistic_update: PatchOperation[]
  loading_state: string           # UI state while processing
  error_handling: inline|toast|modal

# === JSON PATCH FORMAT ===
# IMPORTANT: All collections use MAPS, not arrays
# This enables targeted updates without index shifting

PatchMessage:
  type: "state_patch"
  session_id: uuid
  version: number                 # For ordering
  patches: PatchOperation[]

PatchOperation:
  op: add|remove|replace|copy|move
  path: string                    # JSON Pointer (RFC 6901)
  value: any                      # For add/replace
  from: string                    # For copy/move

# Example: Add message to chat (map-based)
# Path: /chat/channels/incidents/messages/msg-123
# Value: { id: "msg-123", author: {...}, text: "...", timestamp: 123 }

# === UI-ONLY STATE ===
UIState:
  # Local state not synced to backend
  active_instrument: string
  expanded_panels: string[]
  draft_messages: map<channel_id, string>
  scroll_positions: map<instrument_id, number>
  search_queries: map<instrument_id, string>
  selected_items: map<instrument_id, string[]>
  modal_state: { open: boolean, type: string, data: any }

# === EXPRESSION LANGUAGE ===
# For data binding and visibility conditions
Expression:
  # Simple path reference
  "{{path.to.value}}"

  # With filter
  "{{path | filter}}"
  "{{path | filter:arg}}"

  # Ternary
  "{{condition ? trueValue : falseValue}}"

  # Membership
  "{{item.id in selected_items}}"

  # Arithmetic
  "{{alerts | count}}"
  "{{progress * 100}}%"

Filters:
  # Collection
  - count, first, last, filter, sort, slice

  # String
  - uppercase, lowercase, truncate, trim

  # Number
  - round, percent, currency, bytes

  # Date
  - date, time, relative, duration

  # Conditional
  - default, coalesce
```

### Functions
```yaml
Functions:
  # Rendering
  - name: render_component
    signature: "(config: ComponentConfig, context: RenderContext) -> ReactElement"
    description: "Recursively renders component tree from config"

  - name: evaluate_expression
    signature: "(expr: string, context: object) -> any"
    description: "Evaluates template expression against context"

  # State Sync
  - name: apply_patches
    signature: "(state: object, patches: PatchOperation[]) -> object"
    description: "Applies JSON Patch operations to state"

  - name: handle_reconnect
    signature: "(last_version: number) -> PatchOperation[]"
    description: "Get patches since last known version"

  # Actions
  - name: emit_action
    signature: "(action: ActionConfig, context: object) -> void"
    description: "Sends action to backend via WebSocket"

  - name: apply_optimistic_update
    signature: "(patches: PatchOperation[]) -> void"
    description: "Applies optimistic update, tracks for rollback"
```

### Boundaries
- Cannot define validation logic (only reference Layer 3)
- Cannot define evidence extraction
- Cannot define scoring
- Cannot access LLM directly
- Cannot define scenario logic
- State structure mirrors Layer 3 but adds UI-specific fields

---

## Layer 5: Client Portal UI

### Purpose
Defines the client-facing portal for managing assessments, viewing results, and configuring scenarios. Separate from candidate experience.

### Scope
- Candidate management and status
- Assessment reports and analytics
- Scenario configuration interface
- Campaign management
- Organization settings

### Language Constraints
| Allowed | NOT Allowed |
|---------|-------------|
| "Report", "Dashboard", "Analytics" | "JSON Patch" |
| "Campaign", "Cohort", "Batch" | "Instrument", "Widget" |
| "Score breakdown", "Competency chart" | "Event payload" |
| "Scenario template", "Configuration" | "Backbone event" |

### Attributes
```yaml
# === CANDIDATE MANAGEMENT ===
CandidateView:
  candidate_id: uuid
  name: string
  email: string
  status: invited|scheduled|in_progress|completed|expired

  assessments:
    - assessment_id: uuid
      scenario_name: string
      scheduled_at: timestamp
      started_at: timestamp?
      completed_at: timestamp?
      duration: duration?
      status: string

# === ASSESSMENT REPORTS ===
AssessmentReport:
  assessment_id: uuid
  candidate: CandidateView
  scenario: ScenarioSummary

  overall_score: number           # 0-100
  percentile: number              # Compared to population
  recommendation: strong_hire|hire|no_hire|strong_no_hire

  competency_scores:
    - competency: string
      score: number
      percentile: number
      evidence_summary: string
      strengths: string[]
      areas_for_improvement: string[]

  timeline_highlights:            # Key moments
    - timestamp: duration
      description: string
      competency: string
      assessment: positive|negative|neutral

  behavioral_patterns:
    communication_style: string
    decision_making: string
    stress_response: string
    collaboration: string

  detailed_evidence:              # Expandable sections
    - category: string
      items:
        - observable: string
          evidence: string
          strength: string
          timestamp: duration

# === SCENARIO CONFIGURATION ===
ScenarioConfig:
  scenario_id: uuid
  name: string
  description: string
  domain: string
  difficulty: string
  duration_minutes: number

  # Customization options exposed to client
  customization:
    company_name: string          # Replace "TechCorp" with client name
    product_names: string[]       # Customize product references
    team_size: number             # Adjust team references
    tech_stack: string[]          # Customize tech mentions

  competencies_assessed: string[]

  # Preview data (not full scenario script)
  preview:
    narrative_summary: string
    phase_overview: string[]
    sample_challenges: string[]

# === CAMPAIGN MANAGEMENT ===
Campaign:
  campaign_id: uuid
  name: string
  description: string

  scenario_id: uuid

  settings:
    start_date: date
    end_date: date
    time_limit_days: number       # Days to complete after invite
    allow_reschedule: boolean
    max_attempts: number

  candidates:
    total: number
    invited: number
    completed: number
    pending: number

  invitations:
    - candidate_id: uuid
      invited_at: timestamp
      status: pending|opened|started|completed|expired

# === ANALYTICS DASHBOARD ===
AnalyticsDashboard:
  organization_id: uuid

  summary:
    total_assessments: number
    completion_rate: number
    avg_score: number
    score_distribution: map<range, count>

  competency_analysis:
    - competency: string
      avg_score: number
      trend: up|down|stable
      comparison_to_benchmark: number

  scenario_performance:
    - scenario_id: uuid
      scenario_name: string
      assessments_count: number
      avg_score: number
      avg_duration: duration
      completion_rate: number

  time_series:
    - period: string              # "2024-W01"
      assessments: number
      avg_score: number
```

### Functions
```yaml
Functions:
  # Report Generation
  - name: generate_report
    signature: "(assessment_id: uuid) -> AssessmentReport"
    description: "Generate comprehensive assessment report"

  - name: export_report
    signature: "(report: AssessmentReport, format: pdf|csv|json) -> File"
    description: "Export report in requested format"

  # Campaign Management
  - name: create_campaign
    signature: "(config: CampaignConfig) -> Campaign"
    description: "Create new assessment campaign"

  - name: invite_candidates
    signature: "(campaign_id: uuid, candidates: CandidateInfo[]) -> InvitationResult"
    description: "Send invitations to candidates"

  # Analytics
  - name: compute_analytics
    signature: "(org_id: uuid, filters: AnalyticsFilters) -> AnalyticsDashboard"
    description: "Compute analytics for organization"
```

### Boundaries
- Cannot access raw session state
- Cannot see individual events (only aggregated evidence)
- Cannot modify active assessments
- Cannot define scoring formulas (only view results)
- Cannot access scenario scripts (only summaries)

---

## Layer 6: Ultimate Source of Truth (Master Schema)

### Purpose
The complete, authoritative definition of ALL entities, attributes, relationships, integrity rules, and data types across the entire system.

### Scope
- Every entity with every attribute
- All relationships and cardinalities
- Data type definitions
- Integrity constraints
- Cross-layer references
- Computed attribute definitions

### Structure
```yaml
# === ENTITY DEFINITIONS ===

Entity:
  name: string
  description: string
  layer_visibility: Layer[]       # Which layers can access

  attributes:
    - name: string
      type: DataType
      required: boolean
      default: any?
      description: string
      constraints: Constraint[]
      layer_visibility: Layer[]   # Per-attribute visibility

  relationships:
    - name: string
      target: EntityName
      cardinality: one|many
      inverse: string?            # Name of inverse relationship
      required: boolean

  computed_attributes:
    - name: string
      type: DataType
      formula: string
      depends_on: string[]

  integrity_rules:
    - name: string
      rule: string
      error_message: string

# === DATA TYPES ===

DataType:
  primitives:
    - uuid
    - string
    - number
    - boolean
    - timestamp
    - duration
    - date
    - time

  complex:
    - array<T>
    - map<K, V>
    - enum<values>
    - optional<T>
    - union<T1, T2>

  domain:
    - email_address
    - json_pointer
    - expression
    - markdown
    - url

# === CONSTRAINTS ===

Constraint:
  types:
    - required
    - min_length(n)
    - max_length(n)
    - min_value(n)
    - max_value(n)
    - pattern(regex)
    - enum(values)
    - unique
    - unique_within(scope)
    - references(entity.attribute)
    - immutable
    - immutable_after(condition)
```

### Complete Entity Catalog

```yaml
# --- ORGANIZATION DOMAIN ---

Organization:
  layer_visibility: [L5, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: name, type: string, required: true
    - name: slug, type: string, required: true, constraints: [unique, pattern(/^[a-z0-9-]+$/)]
    - name: created_at, type: timestamp, required: true
    - name: settings, type: OrganizationSettings, required: true
  relationships:
    - name: users, target: User, cardinality: many
    - name: campaigns, target: Campaign, cardinality: many
    - name: scenarios, target: Scenario, cardinality: many

User:
  layer_visibility: [L5, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: email, type: email_address, required: true, constraints: [unique]
    - name: name, type: string, required: true
    - name: role, type: enum<admin|manager|viewer>, required: true
    - name: created_at, type: timestamp, required: true
  relationships:
    - name: organization, target: Organization, cardinality: one, required: true

# --- CANDIDATE DOMAIN ---

Candidate:
  layer_visibility: [L3, L5, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: email, type: email_address, required: true
    - name: name, type: string, required: true
    - name: external_id, type: optional<string>
    - name: created_at, type: timestamp, required: true
  relationships:
    - name: organization, target: Organization, cardinality: one, required: true
    - name: sessions, target: Session, cardinality: many

# --- SCENARIO DOMAIN ---

Scenario:
  layer_visibility: [L2, L3, L5, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: name, type: string, required: true
    - name: description, type: markdown, required: true
    - name: domain, type: string, required: true
    - name: difficulty, type: enum<standard|challenging|expert>, required: true
    - name: duration_minutes, type: number, required: true, constraints: [min_value(15), max_value(180)]
    - name: version, type: number, required: true
    - name: status, type: enum<draft|published|archived>, required: true
    - name: created_at, type: timestamp, required: true
    - name: published_at, type: optional<timestamp>
  relationships:
    - name: organization, target: Organization, cardinality: one
    - name: competencies, target: Competency, cardinality: many
    - name: agents, target: AgentPersona, cardinality: many
    - name: event_pool, target: EventDefinition, cardinality: many
    - name: backbone, target: BackboneEntry, cardinality: many
    - name: branches, target: Branch, cardinality: many
    - name: phases, target: Phase, cardinality: many

Competency:
  layer_visibility: [L2, L3, L5, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: name, type: string, required: true
    - name: description, type: markdown, required: true
    - name: category, type: string, required: true
  relationships:
    - name: observables, target: Observable, cardinality: many

Observable:
  layer_visibility: [L2, L3, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: name, type: string, required: true
    - name: description, type: markdown, required: true
    - name: importance, type: enum<critical|high|medium|low>, required: true
  relationships:
    - name: competency, target: Competency, cardinality: one, required: true
    - name: evidence_rules, target: EvidenceRule, cardinality: many

Phase:
  layer_visibility: [L2, L3, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: name, type: string, required: true
    - name: description, type: markdown, required: true
    - name: order, type: number, required: true
    - name: duration_min, type: number, required: true
    - name: duration_max, type: number, required: true
  relationships:
    - name: scenario, target: Scenario, cardinality: one, required: true
    - name: key_observables, target: Observable, cardinality: many

AgentPersona:
  layer_visibility: [L2, L3, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: name, type: string, required: true
    - name: role, type: string, required: true
    - name: personality_traits, type: array<string>, required: true
    - name: communication_style, type: string, required: true
    - name: expertise_areas, type: array<string>, required: true
    - name: response_templates, type: map<string, string>, required: true
    - name: avatar_url, type: optional<url>
  relationships:
    - name: scenario, target: Scenario, cardinality: one, required: true

# --- EVENT POOL ---

EventDefinition:
  layer_visibility: [L2, L3, L6]
  attributes:
    - name: id, type: string, required: true  # Readable ID like "alert_fires"
    - name: name, type: string, required: true
    - name: description, type: markdown, required: true
    - name: event_type, type: string, required: true  # "monitoring/alert_triggered"
    - name: payload_template, type: map<string, any>, required: true
    - name: evidence_category, type: enum<communication|technical|decision|navigation>, required: true
    - name: evidence_importance, type: enum<critical|high|medium|low>, required: true
    - name: tags, type: array<string>, required: true
  relationships:
    - name: scenario, target: Scenario, cardinality: one, required: true
    - name: agent, target: AgentPersona, cardinality: optional  # If agent-generated

BackboneEntry:
  layer_visibility: [L2, L3, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: order, type: number, required: true
    - name: timing_type, type: enum<absolute|relative>, required: true
    - name: timing_value, type: string, required: true  # "start+5m" or "after:event_id+2m"
  relationships:
    - name: scenario, target: Scenario, cardinality: one, required: true
    - name: event, target: EventDefinition, cardinality: one, required: true

Branch:
  layer_visibility: [L2, L3, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: name, type: string, required: true
    - name: description, type: markdown, required: true
    - name: condition, type: BranchCondition, required: true
    - name: priority, type: number, required: true  # For conflict resolution
    - name: cooldown, type: optional<duration>  # Can't re-trigger for this long
  relationships:
    - name: scenario, target: Scenario, cardinality: one, required: true
    - name: events, target: EventDefinition, cardinality: many

BranchCondition:
  layer_visibility: [L3, L6]
  attributes:
    - name: type, type: enum<action_match|state_check|compound>, required: true
    - name: operator, type: optional<enum<and|or|not>>
    - name: action_type, type: optional<string>
    - name: state_path, type: optional<json_pointer>
    - name: comparator, type: optional<enum<eq|ne|gt|lt|gte|lte|contains|matches>>
    - name: value, type: optional<any>
    - name: children, type: optional<array<BranchCondition>>

# --- SESSION DOMAIN ---

Session:
  layer_visibility: [L3, L4, L5, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: status, type: enum<initializing|active|paused|completed|abandoned>, required: true
    - name: started_at, type: timestamp, required: true
    - name: completed_at, type: optional<timestamp>
    - name: scenario_time, type: duration, required: true, default: 0
    - name: time_limit, type: duration, required: true
    - name: current_phase_id, type: uuid, required: true
    - name: backbone_position, type: number, required: true, default: 0
    - name: active_branch_ids, type: array<uuid>, required: true, default: []
    - name: triggered_event_ids, type: array<string>, required: true, default: []
  relationships:
    - name: candidate, target: Candidate, cardinality: one, required: true
    - name: scenario, target: Scenario, cardinality: one, required: true
    - name: events, target: SessionEvent, cardinality: many
    - name: state, target: SessionState, cardinality: one
  computed_attributes:
    - name: duration
      type: duration
      formula: "completed_at ? completed_at - started_at : now() - started_at"
    - name: time_remaining
      type: duration
      formula: "max(0, time_limit - scenario_time)"

SessionEvent:
  layer_visibility: [L3, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: event_type, type: string, required: true
    - name: timestamp, type: timestamp, required: true
    - name: scenario_time, type: duration, required: true
    - name: source, type: enum<candidate|agent|system|timeline>, required: true
    - name: payload, type: map<string, any>, required: true
    - name: evidence_category, type: enum<communication|technical|decision|navigation>, required: true
    - name: evidence_importance, type: enum<critical|high|medium|low>, required: true
    - name: tags, type: array<string>, required: true
    - name: version, type: number, required: true
  relationships:
    - name: session, target: Session, cardinality: one, required: true
  integrity_rules:
    - name: version_monotonic
      rule: "new.version > max(session.events.version)"
      error_message: "Event version must be greater than all existing events"

# --- SESSION STATE (Instrument States) ---

SessionState:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: session_id, type: uuid, required: true
    - name: version, type: number, required: true
    - name: chat, type: ChatState, required: true
    - name: email, type: EmailState, required: true
    - name: monitoring, type: MonitoringState, required: true
    - name: task_board, type: TaskBoardState, required: true
    - name: git_repository, type: GitState, required: true

ChatState:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: channels, type: map<string, ChatChannel>, required: true
    - name: active_channel, type: string, required: true
    - name: unread_counts, type: map<string, number>, required: true
    - name: typing_indicators, type: map<string, array<string>>, required: true

ChatChannel:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: id, type: string, required: true
    - name: name, type: string, required: true
    - name: type, type: enum<channel|dm>, required: true
    - name: members, type: array<string>, required: true
    - name: messages, type: map<string, ChatMessage>, required: true  # Map, not array!

ChatMessage:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: id, type: string, required: true
    - name: sort_key, type: number, required: true  # For ordering in map
    - name: author_id, type: string, required: true
    - name: author_type, type: enum<candidate|agent|system>, required: true
    - name: text, type: string, required: true
    - name: timestamp, type: timestamp, required: true
    - name: edited, type: boolean, required: true, default: false
    - name: reactions, type: map<string, array<string>>, required: true, default: {}

EmailState:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: inbox, type: map<string, Email>, required: true
    - name: sent, type: map<string, Email>, required: true
    - name: drafts, type: map<string, EmailDraft>, required: true
    - name: selected_email_id, type: optional<string>
    - name: active_folder, type: enum<inbox|sent|drafts>, required: true, default: inbox

Email:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: id, type: string, required: true
    - name: sort_key, type: number, required: true
    - name: from, type: string, required: true
    - name: to, type: array<string>, required: true
    - name: cc, type: array<string>, required: true, default: []
    - name: subject, type: string, required: true
    - name: body, type: markdown, required: true
    - name: timestamp, type: timestamp, required: true
    - name: read, type: boolean, required: true, default: false
    - name: thread_id, type: optional<string>
    - name: reply_to_id, type: optional<string>

MonitoringState:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: alerts, type: map<string, Alert>, required: true
    - name: services, type: map<string, ServiceStatus>, required: true
    - name: metrics, type: map<string, MetricSeries>, required: true
    - name: acknowledged_alerts, type: map<string, boolean>, required: true
    - name: active_tab, type: string, required: true, default: overview

Alert:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: id, type: string, required: true
    - name: sort_key, type: number, required: true
    - name: severity, type: enum<critical|warning|info>, required: true
    - name: source, type: string, required: true
    - name: message, type: string, required: true
    - name: timestamp, type: timestamp, required: true
    - name: acknowledged, type: boolean, required: true, default: false
    - name: acknowledged_by, type: optional<string>
    - name: acknowledged_at, type: optional<timestamp>

ServiceStatus:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: id, type: string, required: true
    - name: name, type: string, required: true
    - name: status, type: enum<healthy|degraded|down|unknown>, required: true
    - name: uptime_percent, type: number, required: true
    - name: last_check, type: timestamp, required: true
    - name: dependencies, type: array<string>, required: true

TaskBoardState:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: columns, type: map<string, TaskColumn>, required: true
    - name: tasks, type: map<string, Task>, required: true
    - name: selected_task_id, type: optional<string>
    - name: filter, type: TaskFilter, required: true

TaskColumn:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: id, type: string, required: true
    - name: name, type: string, required: true
    - name: order, type: number, required: true
    - name: task_ids, type: array<string>, required: true  # Ordered list of task IDs
    - name: wip_limit, type: optional<number>

Task:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: id, type: string, required: true
    - name: title, type: string, required: true
    - name: description, type: markdown, required: true
    - name: priority, type: enum<P0|P1|P2|P3>, required: true
    - name: assignee_id, type: optional<string>
    - name: labels, type: array<string>, required: true
    - name: story_points, type: optional<number>
    - name: column_id, type: string, required: true
    - name: created_at, type: timestamp, required: true
    - name: updated_at, type: timestamp, required: true

GitState:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: repositories, type: map<string, Repository>, required: true
    - name: active_repo_id, type: string, required: true
    - name: active_view, type: enum<files|commits|prs|branches>, required: true

Repository:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: id, type: string, required: true
    - name: name, type: string, required: true
    - name: default_branch, type: string, required: true
    - name: branches, type: map<string, GitBranch>, required: true
    - name: commits, type: map<string, GitCommit>, required: true
    - name: pull_requests, type: map<string, PullRequest>, required: true
    - name: files, type: map<string, FileEntry>, required: true

PullRequest:
  layer_visibility: [L3, L4, L6]
  attributes:
    - name: id, type: string, required: true
    - name: number, type: number, required: true
    - name: title, type: string, required: true
    - name: description, type: markdown, required: true
    - name: author_id, type: string, required: true
    - name: status, type: enum<open|merged|closed>, required: true
    - name: source_branch, type: string, required: true
    - name: target_branch, type: string, required: true
    - name: created_at, type: timestamp, required: true
    - name: updated_at, type: timestamp, required: true
    - name: comments, type: map<string, PRComment>, required: true
    - name: reviews, type: map<string, PRReview>, required: true
    - name: files_changed, type: array<string>, required: true
    - name: additions, type: number, required: true
    - name: deletions, type: number, required: true

# --- EVIDENCE & SCORING ---

EvidenceRule:
  layer_visibility: [L3, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: trigger_event_types, type: array<string>, required: true
    - name: extraction_type, type: enum<temporal|behavioral|content|pattern>, required: true
    - name: extraction_params, type: map<string, any>, required: true
    - name: strength_formula, type: expression, required: true
  relationships:
    - name: observable, target: Observable, cardinality: one, required: true

Evidence:
  layer_visibility: [L3, L5, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: strength, type: enum<strong_positive|moderate_positive|weak|moderate_negative|strong_negative>, required: true
    - name: extracted_at, type: timestamp, required: true
    - name: source_event_ids, type: array<uuid>, required: true
    - name: extraction_details, type: map<string, any>, required: true
  relationships:
    - name: session, target: Session, cardinality: one, required: true
    - name: observable, target: Observable, cardinality: one, required: true
    - name: rule, target: EvidenceRule, cardinality: one, required: true

AssessmentResult:
  layer_visibility: [L5, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: overall_score, type: number, required: true, constraints: [min_value(0), max_value(100)]
    - name: recommendation, type: enum<strong_hire|hire|no_hire|strong_no_hire>, required: true
    - name: generated_at, type: timestamp, required: true
  relationships:
    - name: session, target: Session, cardinality: one, required: true
    - name: competency_scores, target: CompetencyScore, cardinality: many

CompetencyScore:
  layer_visibility: [L5, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: score, type: number, required: true, constraints: [min_value(0), max_value(100)]
    - name: evidence_count, type: number, required: true
    - name: summary, type: markdown, required: true
  relationships:
    - name: result, target: AssessmentResult, cardinality: one, required: true
    - name: competency, target: Competency, cardinality: one, required: true
    - name: evidence, target: Evidence, cardinality: many

# --- CAMPAIGN DOMAIN ---

Campaign:
  layer_visibility: [L5, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: name, type: string, required: true
    - name: description, type: markdown, required: true
    - name: status, type: enum<draft|active|paused|completed>, required: true
    - name: start_date, type: date, required: true
    - name: end_date, type: optional<date>
    - name: time_limit_days, type: number, required: true, default: 7
    - name: created_at, type: timestamp, required: true
  relationships:
    - name: organization, target: Organization, cardinality: one, required: true
    - name: scenario, target: Scenario, cardinality: one, required: true
    - name: invitations, target: Invitation, cardinality: many
  computed_attributes:
    - name: total_invited
      type: number
      formula: "count(invitations)"
    - name: total_completed
      type: number
      formula: "count(invitations.filter(i => i.status == 'completed'))"
    - name: completion_rate
      type: number
      formula: "total_invited > 0 ? total_completed / total_invited : 0"

Invitation:
  layer_visibility: [L5, L6]
  attributes:
    - name: id, type: uuid, required: true
    - name: status, type: enum<pending|opened|started|completed|expired>, required: true
    - name: invited_at, type: timestamp, required: true
    - name: opened_at, type: optional<timestamp>
    - name: expires_at, type: timestamp, required: true
    - name: token, type: string, required: true, constraints: [unique]
  relationships:
    - name: campaign, target: Campaign, cardinality: one, required: true
    - name: candidate, target: Candidate, cardinality: one, required: true
    - name: session, target: Session, cardinality: optional
```

### Integrity Rules (Cross-Entity)
```yaml
CrossEntityRules:
  - name: session_scenario_match
    entities: [Session, Scenario]
    rule: "session.current_phase_id in session.scenario.phases.id"
    error: "Session phase must belong to session's scenario"

  - name: backbone_order_unique
    entities: [BackboneEntry]
    rule: "unique(scenario.backbone.order)"
    error: "Backbone entries must have unique order values"

  - name: evidence_observable_scenario
    entities: [Evidence, Observable, Session]
    rule: "evidence.observable.competency in evidence.session.scenario.competencies"
    error: "Evidence observable must belong to session's scenario"

  - name: message_channel_member
    entities: [ChatMessage, ChatChannel]
    rule: "message.author_id in channel.members OR message.author_type == 'system'"
    error: "Message author must be channel member"
```

---

## Layer Relationships

### Data Flow
```
Layer 1 (Requirements)
    ↓ interpreted by
Layer 2 (Domain)
    ↓ implemented by
Layer 3 (Backend) ←→ Layer 4 (Session UI)
    ↓ produces              ↓ syncs via
Layer 6 (Master)        JSON Patch
    ↓ reported in
Layer 5 (Portal)
```

### Cross-Layer References
```yaml
Layer2_to_Layer3:
  - Domain.Competency → Backend.competency_id
  - Domain.Observable → Backend.observable_id
  - Domain.AgentPersona → Backend.agent_id
  - Domain.EventPool → Backend.event_definitions

Layer3_to_Layer4:
  - Backend.SessionState → UI.state (via JSON Patch)
  - Backend.Action.validations → UI.ActionConfig (validation rules)
  - Backend.Event → UI.optimistic_update (rollback source)

Layer3_to_Layer5:
  - Backend.Evidence → Portal.report_evidence
  - Backend.Session → Portal.assessment_status
  - Backend.AssessmentResult → Portal.scores

Layer6_contains_all:
  - Every entity from Layers 2-5
  - Complete attribute definitions
  - All relationships and constraints
```

---

## Usage Guidelines

### For Scenario Authors (Layer 2)
1. Define competencies and observables first
2. Create agent personas with response templates
3. Build event pool with reusable events
4. Define backbone timeline
5. Add branches for reactive behavior
6. Reference Layer 6 for valid attribute names

### For Backend Developers (Layer 3)
1. Implement entities from Layer 6 schema
2. Use Layer 2 domain concepts in business logic
3. Emit events matching event_type patterns
4. Generate patches for Layer 4 consumption
5. Extract evidence using rules

### For Frontend Developers (Layer 4)
1. Build components matching ComponentType registry
2. Bind to state paths from Layer 3
3. Emit actions matching Layer 3 Action types
4. Apply patches using map-based operations
5. Never access Layer 5 data

### For Portal Developers (Layer 5)
1. Display aggregated data from Layer 6
2. Never expose raw events to clients
3. Use computed scores from AssessmentResult
4. Reference scenario summaries, not full scripts

---

## Related Documents

- [Config-Driven UI](./frontend/core/config-driven-ui.md) - Layer 4 implementation
- [Widget Contracts](./frontend/core/widget-contracts.md) - Layer 4 contracts
- [Communication Protocol](./system/communication.md) - Layer 3-4 sync
- [Events Catalog](./reference/events-catalog.md) - Layer 3 events
