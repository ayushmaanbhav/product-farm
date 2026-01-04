# Product-FARM UI Mockups

## Zoom Level Transitions (Google Maps Style)

### Transition: Level 0 → Level 1 (Universe → Product)

```
ZOOM LEVEL 0: UNIVERSE VIEW
════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────┐
│  Product-FARM Dashboard                    🔍 Search   👤 Admin │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ALL PRODUCTS (4)                              Sort: ▼ Health   │
│                                                                 │
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────┐│
│  │ 🟢 Auto Insurance │  │ 🟡 Home Insurance │  │ 🟢 Life Plan   ││
│  │                   │  │                   │  │                ││
│  │    47 rules       │  │    89 rules       │  │    23 rules    ││
│  │   ████████░░      │  │   ██████░░░░      │  │   ██████████   ││
│  │    98.2%          │  │    87.3%          │  │    99.1%       ││
│  │                   │  │                   │  │                ││
│  │  ↓23ms  T1:88%    │  │  ↓156ms T1:67%    │  │  ↓12ms T1:95%  ││
│  └──────────────────┘  └──────────────────┘  └────────────────┘│
│                                                                 │
│  ┌──────────────────┐                                          │
│  │ 🟢 Travel Basic   │       GLOBAL METRICS                    │
│  │                   │       ─────────────                      │
│  │    12 rules       │       Total Rules: 171                   │
│  │   ██████████      │       Avg Health: 96.2%                  │
│  │    99.8%          │       Daily Evals: 12.4M                 │
│  │                   │       Avg Latency: 34ms                  │
│  │  ↓8ms   T1:100%   │                                          │
│  └──────────────────┘                                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

                              │
                              │ 🖱️ Double-click "Auto Insurance"
                              │    or Scroll-zoom on card
                              ▼

ZOOM LEVEL 1: PRODUCT VIEW (Auto Insurance)
════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────┐
│  ◀ All Products    Auto Insurance v2.1         🔍     ⚙️  👤   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────┐ ┌────────────────┐│
│  │                                          │ │ PRODUCT STATS  ││
│  │  TREEMAP VIEW                 [Grid][DAG]│ │                ││
│  │  ┌─────────────────┬─────────────────┐  │ │ Rules:     47  ││
│  │  │                 │                 │  │ │ Attributes:52  ││
│  │  │   CUSTOMER      │    PREMIUMS     │  │ │ DAG Depth:  7  ││
│  │  │   12 attrs      │    15 rules     │  │ │ Parallelism:23 ││
│  │  │   ██████░░░░    │    ████████░░   │  │ │                ││
│  │  │                 │                 │  │ │ ────────────── ││
│  │  ├────────┬────────┼─────────────────┤  │ │ TIER DIST      ││
│  │  │ COVERS │ RISK   │                 │  │ │ T0 ██░░░░ 12%  ││
│  │  │ 8 attr │ 12 rul │   DISCOUNTS     │  │ │ T1 ████████88% ││
│  │  │████████│████░░░░│   10 rules      │  │ │                ││
│  │  │        │        │   ██████████    │  │ │ ────────────── ││
│  │  └────────┴────────┴─────────────────┘  │ │ PERFORMANCE    ││
│  │                                          │ │ Avg:    23ms   ││
│  │  Hover: PREMIUMS (15 rules)             │ │ P99:    142ms  ││
│  │  • base_premium, risk_factor, ...       │ │ Errors: 0.02%  ││
│  │  • Avg latency: 34ms                    │ │                ││
│  │  • 3 LLM rules, 12 deterministic        │ │ [View Metrics] ││
│  └─────────────────────────────────────────┘ └────────────────┘│
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ RECENT EXECUTIONS                          Last 24h ▼      ││
│  │ ▁▂▃▅▇█▇▅▃▂▁▂▃▅▇█▇▅▃▂▁▂▃▅▇█▇▅▃▂▁  12.4M evals  ↓23ms avg  ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

---

### Transition: Level 1 → Level 2 (Product → Rule Graph)

```
                              │
                              │ 🖱️ Double-click "PREMIUMS" group
                              │    or press [DAG] button
                              ▼

ZOOM LEVEL 2: RULE GRAPH VIEW (DAG)
════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────┐
│  ◀ Auto Insurance    Rule Graph                    🔍  [⊞][⊟] │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ EXECUTION LEVELS                      [Timeline] [Hierarchy]│ │
│ │                                                              │ │
│ │ L0 ════════════════════════════════════════════════════════ │ │
│ │     ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐                   │ │
│ │     │ age │   │base │   │claims│   │years│   ← Input attrs  │ │
│ │     │ ░░░ │   │ ░░░ │   │ ░░░ │   │ ░░░ │      (no deps)   │ │
│ │     └──┬──┘   └──┬──┘   └──┬──┘   └──┬──┘                   │ │
│ │        │         │         │         │                       │ │
│ │ L1 ════╪═════════╪═════════╪═════════╪══════════════════════ │ │
│ │        │    ┌────┴────┐    │         │                       │ │
│ │        └────┤risk_fact├────┘         │                       │ │
│ │             │  ▓▓▓▓   │              │                       │ │
│ │             │  892ns  │◄── Hover shows details               │ │
│ │             └────┬────┘              │                       │ │
│ │                  │         ┌─────────┘                       │ │
│ │ L2 ══════════════╪═════════╪════════════════════════════════ │ │
│ │             ┌────┴────┐    │                                 │ │
│ │             │loyalty_ │    │                                 │ │
│ │             │discount │────┤                                 │ │
│ │             │  ░░░    │    │                                 │ │
│ │             └────┬────┘    │                                 │ │
│ │                  │         │                                 │ │
│ │ L3 ══════════════╪═════════╪════════════════════════════════ │ │
│ │        ┌─────────┴─────────┴─────────┐                       │ │
│ │        │      base_premium           │                       │ │
│ │        │         ████                │ ◄── Bytecode (T1)     │ │
│ │        │        330ns                │                       │ │
│ │        └─────────────┬───────────────┘                       │ │
│ │                      │                                       │ │
│ │ L4 ══════════════════╪══════════════════════════════════════ │ │
│ │              ┌───────┴───────┐                               │ │
│ │              │ final_premium │                               │ │
│ │              │    ████░░░░   │                               │ │
│ │              │    1.2μs      │                               │ │
│ │              └───────────────┘                               │ │
│ │                                                              │ │
│ │ ━━━━━━━ Critical path (longest)                             │ │
│ │ ─────── Normal dependency                                    │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ ┌───────────────────────────────────────────────────────────┐   │
│ │ SELECTED: risk_factor                         [View Rule] │   │
│ │ Tier: 1 (Bytecode)  │  Avg: 892ns  │  Evals: 1.2M        │   │
│ │ Inputs: age, claims │  Output: risk_factor               │   │
│ └───────────────────────────────────────────────────────────┘   │
│                                           ┌──────┐              │
│                                           │▪▪▪▪▪▪│ Minimap     │
│                                           │▪┌─┐▪▪│              │
│                                           │▪└─┘▪▪│              │
│                                           └──────┘              │
└─────────────────────────────────────────────────────────────────┘
```

---

### Transition: Level 2 → Level 3 (Graph → Rule Detail)

```
                              │
                              │ 🖱️ Double-click "risk_factor" node
                              │    or press Enter when selected
                              ▼

ZOOM LEVEL 3: RULE DETAIL VIEW
════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────┐
│  ◀ Rule Graph    risk_factor                    T1 ██   🟢 OK  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────────────────────────────────────┐          │
│  │ EXPRESSION                    [FarmScript ▼] [Edit]│         │
│  │ ──────────────────────────────────────────────────│         │
│  │                                                    │         │
│  │  if age < 25 then                                 │         │
│  │      1.5                    ← Young driver factor │         │
│  │  else if age < 35 then                            │         │
│  │      1.2                    ← Standard factor     │         │
│  │  else if claims > 2 then                          │         │
│  │      1.4                    ← Claims penalty      │         │
│  │  else                                             │         │
│  │      1.0                    ← Base factor         │         │
│  │                                                    │         │
│  └──────────────────────────────────────────────────┘          │
│                                                                 │
│  ┌────────────────────────────┐  ┌────────────────────────────┐│
│  │ INPUTS                      │  │ DEPENDENCIES              ││
│  │ ┌────────────────────────┐ │  │                            ││
│  │ │ age         Number     │ │  │    ┌─────┐                 ││
│  │ │ ├─ min: 16             │ │  │ ───│ age │───┐             ││
│  │ │ └─ max: 120            │ │  │    └─────┘   │             ││
│  │ ├────────────────────────┤ │  │              ▼             ││
│  │ │ claims      Number     │ │  │         ┌────────┐         ││
│  │ │ └─ default: 0          │ │  │ ───────►│  this  │         ││
│  │ └────────────────────────┘ │  │         └────────┘         ││
│  │                            │  │              │              ││
│  │ OUTPUT                     │  │    ┌────────┴┐             ││
│  │ ┌────────────────────────┐ │  │    ▼         ▼             ││
│  │ │ risk_factor  Number    │ │  │ ┌─────┐  ┌─────────┐       ││
│  │ │ └─ range: [0.5, 3.0]   │ │  │ │prem │  │discount │       ││
│  │ └────────────────────────┘ │  │ └─────┘  └─────────┘       ││
│  └────────────────────────────┘  └────────────────────────────┘│
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ PERFORMANCE METRICS                                         ││
│  │ ┌───────────────────────────────────────────────────────┐  ││
│  │ │ LATENCY DISTRIBUTION (last 1M evaluations)            │  ││
│  │ │                                                        │  ││
│  │ │     ▁▂▃▅▇█▇▅▃▂▁░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │  ││
│  │ │     │         │         │         │         │         │  ││
│  │ │   100ns     500ns      1μs       5μs      10μs        │  ││
│  │ │                                                        │  ││
│  │ │ avg: 892ns │ p50: 780ns │ p99: 2.1μs │ max: 8.3μs    │  ││
│  │ └───────────────────────────────────────────────────────┘  ││
│  │                                                             ││
│  │ ┌─────────────┬─────────────┬─────────────┬─────────────┐  ││
│  │ │ Evaluations │ Cache Hits  │ Tier Promos │ Errors      │  ││
│  │ │  1,247,892  │   99.2%     │    0        │   0 (0%)    │  ││
│  │ └─────────────┴─────────────┴─────────────┴─────────────┘  ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ TEST SCENARIOS                              [+ Add] [Run All]││
│  │ ┌───────────────────────────────────────────────────────┐  ││
│  │ │ Test              │ Input              │ Expected │ ✓  │  ││
│  │ ├───────────────────┼────────────────────┼──────────┼────┤  ││
│  │ │ 🟢 Young driver    │ age=20, claims=0   │ 1.5      │ ✓  │  ││
│  │ │ 🟢 Middle aged     │ age=30, claims=0   │ 1.2      │ ✓  │  ││
│  │ │ 🟢 Senior safe     │ age=55, claims=0   │ 1.0      │ ✓  │  ││
│  │ │ 🟢 Claims penalty  │ age=40, claims=3   │ 1.4      │ ✓  │  ││
│  │ └───────────────────────────────────────────────────────┘  ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                 │
│  [View Bytecode]   [View AST]   [Compare Versions]   [History] │
└─────────────────────────────────────────────────────────────────┘
```

---

### Transition: Level 3 → Level 4 (Rule → Bytecode Trace)

```
                              │
                              │ 🖱️ Click [View Bytecode] button
                              │    or press 'B' key
                              ▼

ZOOM LEVEL 4: BYTECODE EXECUTION TRACE
════════════════════════════════════════════════════════════════════

┌─────────────────────────────────────────────────────────────────┐
│  ◀ risk_factor    Bytecode Trace            [Step] [Run] [Reset]│
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  TEST INPUT                                                     │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ { "age": 30, "claims": 1 }                    [Edit] [Run] ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                 │
│  EXECUTION TRACE                                                │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ IP  │ OpCode     │ Operands   │ Stack          │ Time │ Br ││
│  │─────┼────────────┼────────────┼────────────────┼──────┼────││
│  │  0  │ LoadVar    │ age        │ [30]           │  12ns│    ││
│  │  1  │ LoadConst  │ 25         │ [30, 25]       │   8ns│    ││
│  │  2  │ Lt         │            │ [false]        │  15ns│    ││
│  │  3  │ JumpIfNot  │ @7         │ []             │   5ns│ →7 ││
│  │  ─  │ ─ ─ ─ ─ ─  │ ─ ─ ─ ─ ─  │ ─ ─ ─ ─ ─ ─ ─ │ ─ ─  │skip││
│  │  7  │ LoadVar    │ age        │ [30]           │  12ns│    ││
│  │  8  │ LoadConst  │ 35         │ [30, 35]       │   8ns│    ││
│  │► 9  │ Lt         │            │ [true]         │  15ns│    ││ ◄─ CURRENT
│  │ 10  │ JumpIfNot  │ @14        │ []             │   5ns│    ││
│  │ 11  │ LoadConst  │ 1.2        │ [1.2]          │   8ns│    ││
│  │ 12  │ Jump       │ @END       │ [1.2]          │   5ns│    ││
│  │ END │ Return     │            │ ══► 1.2        │   3ns│    ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                 │
│  ┌────────────────────────────┐  ┌────────────────────────────┐│
│  │ CONTROL FLOW               │  │ VARIABLE STATE             ││
│  │                            │  │                            ││
│  │    ┌──────────────┐        │  │ age    = 30                ││
│  │    │ age < 25 ?   │        │  │ claims = 1                 ││
│  │    └──────┬───────┘        │  │                            ││
│  │       NO  │                │  │ ─────────────────────────  ││
│  │           ▼                │  │                            ││
│  │    ┌──────────────┐        │  │ CONSTANT POOL              ││
│  │    │ age < 35 ?   │ ◄──    │  │ [0]: 25                    ││
│  │    └──────┬───────┘ HERE   │  │ [1]: 35                    ││
│  │      YES  │                │  │ [2]: 1.5                   ││
│  │           ▼                │  │ [3]: 1.2                   ││
│  │    ┌──────────────┐        │  │ [4]: 2                     ││
│  │    │ return 1.2   │        │  │ [5]: 1.4                   ││
│  │    └──────────────┘        │  │ [6]: 1.0                   ││
│  └────────────────────────────┘  └────────────────────────────┘│
│                                                                 │
│  EXECUTION SUMMARY                                              │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ Instructions:  8 executed  │  6 skipped (branch not taken) ││
│  │ Total Time:    91ns        │  Branch: else-if (age < 35)   ││
│  │ Result:        1.2         │  Type: Number                 ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                 │
│  [◄ Prev Step]  [Next Step ►]  [Run to End]  [Reset]           │
└─────────────────────────────────────────────────────────────────┘
```

---

## Semantic Zoom Behavior

### Information Density by Zoom Level

```
ZOOM LEVEL │ WHAT'S SHOWN
───────────┼────────────────────────────────────────────────────────
    0      │ Product cards with: name, rule count, health %, mini sparkline
    ↓      │
    1      │ Attribute groups: name, item count, health bar, top metrics
    ↓      │ Expand: show individual attrs/rules as small chips
    2      │ DAG nodes: name, tier badge, latency, dependency arrows
    ↓      │ Hover: input/output summary, last execution
    3      │ Full rule: expression, all inputs/outputs, histograms, tests
    ↓      │ Tabs: expression views, performance deep-dive
    4      │ Bytecode: instruction-by-instruction trace, stack, registers
           │ Step debugger, branch visualization
```

### Text Truncation Examples

```
FULL NAME: premium_calculation_for_young_drivers_with_claims_history

Level 0:  "premium_ca..."           (12 chars max)
Level 1:  "premium_calculation..."  (24 chars max)
Level 2:  "premium_calculation_for_young..." (36 chars max)
Level 3:  Full name shown
Level 4:  Full name + description

Hover at any level → Show full name in tooltip
```

---

## Interactive Elements Summary

### Click Behaviors by Level

```
┌─────────┬────────────────┬─────────────────────────────────────┐
│ Level   │ Single Click   │ Double Click                        │
├─────────┼────────────────┼─────────────────────────────────────┤
│ 0       │ Select product │ Zoom to Level 1 (Product View)      │
│ 1       │ Select group   │ Zoom to Level 2 (Rule Graph)        │
│ 2       │ Select node    │ Zoom to Level 3 (Rule Detail)       │
│ 3       │ Select section │ Expand section / Edit mode          │
│ 4       │ Select instr   │ Set breakpoint                      │
└─────────┴────────────────┴─────────────────────────────────────┘
```

### Keyboard Navigation

```
┌─────────────────────────────────────────────────────────────────┐
│ UNIVERSAL SHORTCUTS                                             │
├─────────────────────────────────────────────────────────────────┤
│ + / - / Scroll     Zoom in / out                                │
│ Arrow Keys         Navigate between items                       │
│ Enter              Zoom into selected / Expand                  │
│ Backspace / Esc    Zoom out / Go back                          │
│ Space + Drag       Pan view                                     │
│ F                  Fit all to view                              │
│ 1-4                Jump to zoom level                           │
│ Cmd+F              Search                                       │
│ /                  Quick command palette                        │
│ ?                  Show shortcuts help                          │
├─────────────────────────────────────────────────────────────────┤
│ LEVEL 2 (DAG) SPECIFIC                                          │
├─────────────────────────────────────────────────────────────────┤
│ L                  Toggle level lines                           │
│ T                  Toggle timing overlay                        │
│ P                  Highlight critical path                      │
│ H                  Toggle heatmap mode                          │
│ M                  Toggle minimap                               │
│ Tab                Cycle through connected nodes                │
├─────────────────────────────────────────────────────────────────┤
│ LEVEL 4 (TRACE) SPECIFIC                                        │
├─────────────────────────────────────────────────────────────────┤
│ N / Space          Next instruction                             │
│ P                  Previous instruction                         │
│ R                  Run to end                                   │
│ S                  Step into                                    │
│ O                  Step over                                    │
│ B                  Toggle breakpoint                            │
└─────────────────────────────────────────────────────────────────┘
```

---

## Color Coding Reference

```
HEALTH INDICATORS
🟢 Green  (#22C55E)  │ Healthy: >95% success, <100ms p99
🟡 Yellow (#EAB308)  │ Warning: 90-95% success, 100-500ms p99
🔴 Red    (#EF4444)  │ Critical: <90% success, >500ms p99

TIER BADGES
░░ Gray   (#9CA3AF)  │ Tier 0 - AST interpretation
██ Blue   (#3B82F6)  │ Tier 1 - Bytecode compiled
▓▓ Purple (#8B5CF6)  │ Tier 2 - JIT compiled (future)

RULE TYPES
⚡ Lightning         │ Deterministic (JSON Logic)
🤖 Robot            │ LLM-powered rule
📐 Square           │ Custom evaluator

EXECUTION STATUS
○  Empty circle     │ Pending
●  Filled circle    │ Running (animated pulse)
✓  Checkmark        │ Completed successfully
✗  X mark           │ Failed
↺  Circular arrow   │ Cached result used
```

---

## Responsive Breakpoints

```
DESKTOP (≥1280px)
┌────────────────────────────────────────────────────────────────┐
│ Sidebar │         Main Content Area          │ Details Panel  │
│  240px  │            flexible               │     320px      │
└────────────────────────────────────────────────────────────────┘

TABLET (768px - 1279px)
┌────────────────────────────────────────────────────────────────┐
│ [≡]  │              Main Content Area                         │
│      │    (Sidebar collapsed, Details as bottom sheet)        │
└────────────────────────────────────────────────────────────────┘

MOBILE (<768px)
┌────────────────────────────────────────────────────────────────┐
│                    Single Panel View                           │
│              (Full-screen, swipe navigation)                   │
│              [◀ Back]              [Details ▼]                │
└────────────────────────────────────────────────────────────────┘
```
