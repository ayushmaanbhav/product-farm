# Product-FARM Visualization System Design

## Vision: "Google Maps for Rule Engines"

A multi-level, zoomable interface that reveals progressive detail - from universe overview to individual bytecode instructions - with intelligent information density management at every level.

---

## Core Design Principles

### 1. Progressive Disclosure
Like Google Maps showing continents → countries → cities → streets → buildings, our UI reveals:
- **Zoom Out**: Aggregated metrics, patterns, health indicators
- **Zoom In**: Individual details, specific values, execution traces

### 2. Semantic Zoom
Content transforms based on zoom level - not just scales:
- At 10,000ft: Show "47 rules, 99.2% healthy"
- At 1,000ft: Show rule categories and flow
- At 100ft: Show individual rules with expressions
- At 10ft: Show bytecode, timing, variable traces

### 3. Information Density Balance
- **Constant cognitive load** at every zoom level
- Smart text truncation with expand-on-hover
- Contextual metrics (show what matters at this level)

---

## Zoom Levels Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  LEVEL 0: UNIVERSE                                              │
│  "All Products Overview"                                        │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐                              │
│  │ 47  │ │ 23  │ │ 89  │ │ 12  │  ← Product cards with       │
│  │rules│ │rules│ │rules│ │rules│    health indicators         │
│  └─────┘ └─────┘ └─────┘ └─────┘                              │
├─────────────────────────────────────────────────────────────────┤
│  LEVEL 1: PRODUCT                                               │
│  "Single Product - Attribute Groups"                            │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐              │
│  │  Customer   │ │   Covers    │ │  Premiums   │              │
│  │  12 attrs   │ │   8 attrs   │ │  27 rules   │              │
│  │  ████░░░░   │ │  █████████  │ │  ██████░░░  │              │
│  └─────────────┘ └─────────────┘ └─────────────┘              │
├─────────────────────────────────────────────────────────────────┤
│  LEVEL 2: RULE GRAPH                                            │
│  "DAG Visualization with Execution Flow"                        │
│                    ┌───┐                                        │
│              ┌────►│ B │────┐                                  │
│  ┌───┐      │     └───┘     │      ┌───┐                      │
│  │ A │──────┤               ├─────►│ D │                       │
│  └───┘      │     ┌───┐     │      └───┘                      │
│              └────►│ C │────┘                                  │
│                    └───┘                                        │
├─────────────────────────────────────────────────────────────────┤
│  LEVEL 3: RULE DETAIL                                           │
│  "Individual Rule - Expression & Metrics"                       │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ premium_calculation                          Tier 1 ██  │   │
│  │ ─────────────────────────────────────────────────────── │   │
│  │ if age < 25 then base * 1.5                             │   │
│  │ else if age < 35 then base * 1.2                        │   │
│  │ else base                                               │   │
│  │ ─────────────────────────────────────────────────────── │   │
│  │ Inputs: age, base  │  Output: premium  │  330ns avg     │   │
│  └─────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│  LEVEL 4: EXECUTION TRACE                                       │
│  "Bytecode & Runtime Analysis"                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 0: LoadVar   age     → R0 = 30                          │   │
│  │ 1: LoadConst 25      → R1 = 25                          │   │
│  │ 2: Lt        R0, R1  → R2 = false                       │   │
│  │ 3: JumpIf    R2, 6   → skip (false)                     │   │
│  │ 4: LoadVar   age     → R0 = 30                          │   │
│  │ 5: LoadConst 35      → R1 = 35    ◄── Currently here    │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Level 0: Universe View

### What's Visible
- All products as cards/tiles
- Health status (green/yellow/red)
- Rule count badges
- Last execution timestamp
- Quick metrics sparklines

### Metrics Shown
| Metric | Visualization |
|--------|---------------|
| Total Rules | Large number |
| Health Score | Color-coded ring |
| Avg Execution Time | Mini bar |
| Active/Draft/Archived | Stacked indicator |
| LLM vs Deterministic | Ratio badge |

### Information Density
```
┌────────────────────────┐
│ ● Auto Insurance    47 │  ← Name + rule count
│   ████████░░  98.2%    │  ← Health bar + percentage
│   ↓23ms  ↑99.9%  T:3   │  ← Latency, uptime, tier distribution
└────────────────────────┘
```

### Interactions
- **Click**: Zoom to Level 1 (Product)
- **Hover**: Show tooltip with recent metrics
- **Right-click**: Context menu (clone, archive, compare)
- **Drag**: Rearrange products
- **Ctrl+Click**: Multi-select for comparison

---

## Level 1: Product View

### Layout Options

#### 1.1 Sunburst View (Radial)
```
                    ┌─────────┐
              ┌─────┤ Product ├─────┐
              │     └─────────┘     │
        ┌─────┴─────┐         ┌─────┴─────┐
        │ Customer  │         │  Covers   │
        └─────┬─────┘         └─────┬─────┘
         ┌────┴────┐           ┌────┴────┐
       ┌─┴─┐    ┌──┴──┐     ┌──┴──┐   ┌──┴──┐
       │age│    │name │     │fire │   │flood│
       └───┘    └─────┘     └─────┘   └─────┘
```

#### 1.2 Treemap View (Space-Filling)
```
┌───────────────────────────────────────────┐
│ Customer (12)           │ Premiums (27)   │
│ ┌─────┬─────┬─────────┐ │ ┌─────────────┐ │
│ │ age │name │ address │ │ │base_premium │ │
│ ├─────┴─────┼─────────┤ │ ├─────────────┤ │
│ │   email   │  phone  │ │ │risk_factor  │ │
│ └───────────┴─────────┘ │ └─────────────┘ │
├─────────────────────────┼─────────────────┤
│ Covers (8)              │ Signals (5)     │
│ ┌───────────┬─────────┐ │ ┌─────────────┐ │
│ │   fire    │  flood  │ │ │ fraud_score │ │
│ └───────────┴─────────┘ │ └─────────────┘ │
└─────────────────────────┴─────────────────┘
```

### Metrics Panel (Always Visible)
```
┌─────────────────────────────────────────┐
│ Auto Insurance v2.1                     │
├─────────────────────────────────────────┤
│ Attributes: 47   │ Rules: 89           │
│ DAG Depth: 7     │ Max Parallel: 23    │
│ Tier 0: 12%      │ Tier 1: 88%         │
│ Avg Latency: 23ms│ P99: 142ms          │
│ LLM Rules: 3     │ Deterministic: 86   │
└─────────────────────────────────────────┘
```

### Interactions
- **Scroll wheel**: Zoom to Level 2 (focused area)
- **Click group**: Expand to show children
- **Double-click**: Jump to Level 2 (Rule Graph)
- **Tab**: Navigate between groups
- **Cmd+F**: Search attributes/rules

---

## Level 2: Rule Graph View

### DAG Visualization Modes

#### 2.1 Hierarchical (Top-Down Execution Flow)
```
Level 0 ─────────────────────────────────────────────────
         ┌─────────┐  ┌─────────┐  ┌─────────┐
         │  age    │  │  base   │  │ history │
         └────┬────┘  └────┬────┘  └────┬────┘
              │            │            │
Level 1 ──────┼────────────┼────────────┼────────────────
              │      ┌─────┴─────┐      │
              └──────┤risk_factor├──────┘
                     └─────┬─────┘
                           │
Level 2 ───────────────────┼─────────────────────────────
              ┌────────────┴────────────┐
              │                         │
         ┌────┴────┐              ┌─────┴─────┐
         │premium_a│              │premium_b  │
         └────┬────┘              └─────┬─────┘
              │                         │
Level 3 ──────┴─────────────────────────┴────────────────
                     ┌─────────┐
                     │  total  │
                     └─────────┘
```

#### 2.2 Force-Directed (Organic Clustering)
```
            ○ age
           ╱
      ○───○ risk_factor ───○ premium_a
     ╱                    ╲
○ base                     ○ total
     ╲                    ╱
      ○───○ discount  ────○ premium_b
           ╲
            ○ loyalty
```

#### 2.3 Execution Timeline (Gantt-style)
```
Time →   0ms      50ms     100ms    150ms    200ms
         │        │        │        │        │
age      ████
base     ████
history  ██████
         │        │        │        │        │
risk     ░░░░░░░░░████
discount ░░░░░░░░░██████
         │        │        │        │        │
premium_a░░░░░░░░░░░░░░░░░░████
premium_b░░░░░░░░░░░░░░░░░░██████
         │        │        │        │        │
total    ░░░░░░░░░░░░░░░░░░░░░░░░░░████
```

### Node Information Density

#### Zoomed Out (Many Nodes Visible)
```
┌──────┐
│ risk │  ← Just name
│  ●   │  ← Health indicator
└──────┘
```

#### Mid-Zoom
```
┌─────────────────┐
│ risk_factor     │
│ T1 │ 330ns │ ●  │  ← Tier, latency, health
│ 3→1             │  ← Inputs→Outputs count
└─────────────────┘
```

#### Zoomed In
```
┌─────────────────────────────────────┐
│ risk_factor                    T1 ● │
├─────────────────────────────────────┤
│ if age < 25 then 1.5                │
│ else if age < 35 then 1.2           │
│ else 1.0                            │
├─────────────────────────────────────┤
│ Inputs:  age, claims_history        │
│ Output:  risk_factor                │
│ Timing:  avg 330ns │ p99 1.2μs      │
│ Evals:   1,247,892 │ Errors: 0      │
└─────────────────────────────────────┘
```

### Edge Information
```
─────────────────────►  Normal dependency
═════════════════════►  Critical path (slowest)
─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ►  Optional/conditional
━━━━━━━━━━━━━━━━━━━━►  High-traffic (hot path)
```

### Interactions
- **Scroll wheel**: Zoom in/out (semantic)
- **Click node**: Select and show details panel
- **Double-click**: Zoom to Level 3 (Rule Detail)
- **Drag**: Pan the graph
- **Shift+Drag**: Box select multiple nodes
- **Space+Drag**: Pan (Google Maps style)
- **Arrow keys**: Navigate between connected nodes
- **L**: Toggle level lines
- **T**: Toggle timing overlay
- **P**: Highlight critical path

---

## Level 3: Rule Detail View

### Layout
```
┌─────────────────────────────────────────────────────────────────┐
│ ◄ Back to Graph                              risk_factor    T1 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  EXPRESSION (FarmScript)                          [Toggle AST] │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ if age < 25 then                                          │ │
│  │     1.5                                                   │ │
│  │ else if age < 35 then                                     │ │
│  │     1.2                                                   │ │
│  │ else                                                      │ │
│  │     1.0                                                   │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌──────────────────┐  ┌──────────────────────────────────────┐│
│  │ INPUTS           │  │ DEPENDENCIES                         ││
│  │ ┌──────────────┐ │  │                                      ││
│  │ │ age: Number  │ │  │      ┌─────┐                        ││
│  │ │ min: 0       │ │  │  ────┤ age ├────►[this]             ││
│  │ │ max: 150     │ │  │      └─────┘                        ││
│  │ └──────────────┘ │  │                                      ││
│  └──────────────────┘  └──────────────────────────────────────┘│
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ METRICS                                                   │  │
│  │ ┌────────────┬────────────┬────────────┬────────────────┐│  │
│  │ │ Avg Time   │ P50        │ P99        │ Evaluations    ││  │
│  │ │ 330ns      │ 285ns      │ 1.2μs      │ 1,247,892      ││  │
│  │ └────────────┴────────────┴────────────┴────────────────┘│  │
│  │                                                           │  │
│  │ Latency Distribution                                      │  │
│  │ ▁▂▃▅▇█▇▅▃▂▁░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │  │
│  │ 100ns                                              10μs   │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ TEST SCENARIOS                              [+ Add Test] │  │
│  │ ┌────────────────────────────────────────────────────────┐│  │
│  │ │ ● Young driver    │ age=20 → 1.5           │ 285ns    ││  │
│  │ │ ● Middle aged     │ age=30 → 1.2           │ 312ns    ││  │
│  │ │ ● Senior          │ age=65 → 1.0           │ 298ns    ││  │
│  │ └────────────────────────────────────────────────────────┘│  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  [View Bytecode]  [View AST]  [Run Test]  [Edit Rule]          │
└─────────────────────────────────────────────────────────────────┘
```

### Expression Views

#### FarmScript (Human-Readable)
```
if age < 25 then 1.5
else if age < 35 then 1.2
else 1.0
```

#### JSON Logic
```json
{
  "if": [
    {"<": [{"var": "age"}, 25]}, 1.5,
    {"<": [{"var": "age"}, 35]}, 1.2,
    1.0
  ]
}
```

#### AST Tree
```
If
├── Condition: Lt
│   ├── Var("age")
│   └── Const(25)
├── Then: Const(1.5)
└── Else: If
    ├── Condition: Lt
    │   ├── Var("age")
    │   └── Const(35)
    ├── Then: Const(1.2)
    └── Else: Const(1.0)
```

### Interactions
- **Tab**: Switch between expression views
- **Click metric**: Expand detailed breakdown
- **Click test**: Run and show trace
- **E**: Edit mode
- **R**: Run all tests
- **B**: View bytecode (Level 4)

---

## Level 4: Execution Trace View

### Bytecode Viewer
```
┌─────────────────────────────────────────────────────────────────┐
│ BYTECODE TRACE                                    [Step] [Run] │
├─────────────────────────────────────────────────────────────────┤
│ Test Input: { age: 30 }                                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  IP │ OpCode    │ Args      │ Stack State        │ Time       │
│ ────┼───────────┼───────────┼────────────────────┼────────────│
│   0 │ LoadVar   │ age       │ [30]               │ 12ns       │
│   1 │ LoadConst │ 25        │ [30, 25]           │  8ns       │
│   2 │ Lt        │           │ [false]            │ 15ns       │
│   3 │ JumpIfNot │ @7        │ []                 │  5ns       │
│ → 7 │ LoadVar   │ age       │ [30]               │ 12ns  ◄──  │
│   8 │ LoadConst │ 35        │ [30, 35]           │  8ns       │
│   9 │ Lt        │           │ [true]             │ 15ns       │
│  10 │ JumpIfNot │ @14       │ []                 │  5ns       │
│  11 │ LoadConst │ 1.2       │ [1.2]              │  8ns       │
│  12 │ Return    │           │ Result: 1.2        │  3ns       │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│ REGISTERS                                                       │
│ R0: 30 (age)    R1: 35    R2: true    R3: 1.2                  │
├─────────────────────────────────────────────────────────────────┤
│ CONSTANT POOL                                                   │
│ [0]: 25  [1]: 35  [2]: 1.5  [3]: 1.2  [4]: 1.0                 │
├─────────────────────────────────────────────────────────────────┤
│ EXECUTION SUMMARY                                               │
│ Instructions: 8 executed, 6 skipped                             │
│ Total Time: 91ns                                                │
│ Branch taken: else-if (age < 35)                                │
└─────────────────────────────────────────────────────────────────┘
```

### Control Flow Visualization
```
     ┌──────────────┐
     │ age < 25 ?   │
     └──────┬───────┘
        NO  │
            ▼
     ┌──────────────┐
     │ age < 35 ?   │ ◄── CURRENT
     └──────┬───────┘
       YES  │
            ▼
     ┌──────────────┐
     │ return 1.2   │
     └──────────────┘
```

---

## Information Density Management

### Text Truncation Strategy

```
Full Text:           "premium_calculation_for_young_drivers_with_claims"
Level 0 (Universe):  "premium_calc..."
Level 1 (Product):   "premium_calculation_for_young..."
Level 2 (Graph):     "premium_calculation_for_young_drivers..."
Level 3 (Detail):    "premium_calculation_for_young_drivers_with_claims"
```

### Smart Grouping

When too many items exist at a level:
```
Before (30 rules):
┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ... ┌─────┐
│rule1│ │rule2│ │rule3│ │rule4│ │rule5│     │rule30│
└─────┘ └─────┘ └─────┘ └─────┘ └─────┘ ... └─────┘

After (auto-grouped):
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│ Premium Rules │ │ Risk Rules    │ │ Discount Rules│
│ (12 rules)    │ │ (8 rules)     │ │ (10 rules)    │
│ ████████░░░░  │ │ ██████████░░  │ │ ████████████  │
└───────────────┘ └───────────────┘ └───────────────┘
```

### Metric Adaptation by Zoom Level

| Zoom Level | Metrics Shown |
|------------|---------------|
| 0 (Universe) | Total rules, health %, single latency number |
| 1 (Product) | Group counts, tier distribution, top 3 slowest |
| 2 (Graph) | Per-rule latency, dependency count, eval count |
| 3 (Detail) | Full histogram, percentiles, error rates |
| 4 (Trace) | Per-instruction timing, branch prediction |

---

## Navigation & Shortcuts

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `+` / `-` or `Scroll` | Zoom in/out |
| `Arrow keys` | Navigate nodes/items |
| `Enter` | Expand/zoom into selected |
| `Backspace` / `Esc` | Zoom out / go back |
| `Space` + `Drag` | Pan view |
| `F` | Fit all to view |
| `1-4` | Jump to zoom level |
| `Cmd+F` | Search |
| `Cmd+G` | Go to rule by ID |
| `/` | Quick command palette |
| `M` | Toggle minimap |
| `L` | Toggle level lines |
| `T` | Toggle timing overlay |
| `P` | Highlight critical path |
| `H` | Toggle heatmap mode |
| `?` | Show all shortcuts |

### Mouse Interactions

| Action | Result |
|--------|--------|
| Click | Select |
| Double-click | Zoom in / expand |
| Right-click | Context menu |
| Scroll | Zoom (semantic) |
| Shift+Scroll | Horizontal scroll |
| Ctrl+Scroll | Zoom (geometric) |
| Drag | Pan |
| Shift+Drag | Box select |
| Middle-click+Drag | Free pan |

### Touch Gestures (Tablet/Mobile)

| Gesture | Action |
|---------|--------|
| Tap | Select |
| Double-tap | Zoom in |
| Pinch | Zoom in/out |
| Two-finger drag | Pan |
| Long-press | Context menu |
| Swipe left | Go back |

---

## Specialized Views

### 1. Comparison View
```
┌─────────────────────────────────────────────────────────────────┐
│ COMPARE: v2.0 vs v2.1                                           │
├────────────────────────────┬────────────────────────────────────┤
│ v2.0                       │ v2.1                               │
├────────────────────────────┼────────────────────────────────────┤
│ Rules: 47                  │ Rules: 52 (+5)                     │
│ Avg Latency: 28ms          │ Avg Latency: 23ms (-18%)          │
│ Tier 1: 78%                │ Tier 1: 88% (+10%)                 │
├────────────────────────────┼────────────────────────────────────┤
│ if age < 25 then           │ if age < 25 then                   │
│     base * 1.5             │     base * 1.6  ◄── CHANGED        │
│ else base                  │ else if age < 30 then ◄── NEW      │
│                            │     base * 1.3                     │
│                            │ else base                          │
└────────────────────────────┴────────────────────────────────────┘
```

### 2. Execution Replay View
```
┌─────────────────────────────────────────────────────────────────┐
│ EXECUTION REPLAY                     [◄◄] [◄] [▶] [►►] [Speed] │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ Time: 23ms / 156ms                    ████████░░░░░░░░░░░░░░░  │
│                                                                 │
│ Currently Executing:                                            │
│ ┌─────────────────────────────────────────────────────────────┐│
│ │ Level 3: premium_calculation                                ││
│ │ Input:  { age: 30, base: 100 }                              ││
│ │ Output: { premium: 120 }  ◄── Computing...                  ││
│ └─────────────────────────────────────────────────────────────┘│
│                                                                 │
│ Execution Stack:                                                │
│ ✓ Level 0: age, base_rate, history (3 rules)         [12ms]   │
│ ✓ Level 1: risk_factor (1 rule)                      [8ms]    │
│ ✓ Level 2: discount_factor (1 rule)                  [3ms]    │
│ ► Level 3: premium_calculation (1 rule)              [...]    │
│ ○ Level 4: final_price (1 rule)                      [pending]│
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3. Heatmap View (Performance Analysis)
```
┌─────────────────────────────────────────────────────────────────┐
│ EXECUTION HEATMAP                         [By Time] [By Count] │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  < 100ns (fast)             │
│   ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  100ns - 1μs               │
│   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  1μs - 10μs                 │
│   ████████████████████████████████  > 10μs (slow)              │
│                                                                 │
│   ┌───┐ ┌───┐ ┌───┐                                           │
│   │░░░│ │░░░│ │▒▒▒│  Level 0                                  │
│   └─┬─┘ └─┬─┘ └─┬─┘                                           │
│     │     │     │                                              │
│   ┌─┴─────┴─────┴─┐                                           │
│   │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  Level 1  ◄── Bottleneck!                │
│   └───────┬───────┘                                           │
│           │                                                    │
│   ┌───────┴───────┐                                           │
│   │░░░░░░░░░░░░░░░│  Level 2                                  │
│   └───────────────┘                                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 4. LLM Rules Dashboard
```
┌─────────────────────────────────────────────────────────────────┐
│ LLM RULES OVERVIEW                                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ Provider Distribution          Cost Analysis (Last 24h)        │
│ ┌─────────────────────┐       ┌─────────────────────────────┐  │
│ │ Claude ████████ 67% │       │ Total Calls:     12,847     │  │
│ │ Ollama ████░░░░ 33% │       │ Total Cost:      $23.47     │  │
│ └─────────────────────┘       │ Avg Latency:     847ms      │  │
│                               │ Error Rate:      0.3%       │  │
│ Rule Performance              └─────────────────────────────┘  │
│ ┌───────────────────────────────────────────────────────────┐  │
│ │ Rule                    │ Calls │ Latency │ Cost   │ Errs │  │
│ │ fraud_detection         │ 5,234 │  923ms  │ $12.34 │  12  │  │
│ │ sentiment_analysis      │ 4,102 │  654ms  │  $8.21 │   3  │  │
│ │ document_classification │ 3,511 │  1.2s   │  $2.92 │   0  │  │
│ └───────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Minimap & Overview

### Always-Visible Minimap
```
┌────────────────┐
│ ▪▪▪▪▪▪▪▪▪▪▪▪  │  ← Full product graph
│ ▪▪▪▪┌───┐▪▪▪  │
│ ▪▪▪▪│   │▪▪▪  │  ← Current viewport
│ ▪▪▪▪└───┘▪▪▪  │
│ ▪▪▪▪▪▪▪▪▪▪▪▪  │
└────────────────┘
```

### Breadcrumb Navigation
```
Universe > Auto Insurance v2.1 > Premium Rules > risk_factor > Bytecode
   ▲            ▲                    ▲              ▲            ▲
 Level 0     Level 1              Level 2       Level 3      Level 4
```

---

## Responsive Design

### Desktop (1920x1080+)
- Full sidebar + main view + details panel
- Minimap always visible
- Full keyboard shortcuts

### Tablet (768px - 1024px)
- Collapsible sidebar
- Touch-optimized nodes (larger hit targets)
- Swipe gestures for navigation

### Mobile (< 768px)
- Single-panel view with back navigation
- Bottom sheet for details
- Simplified metrics display

---

## Color System

### Semantic Colors
| Color | Meaning |
|-------|---------|
| Green (#22C55E) | Healthy, fast, success |
| Yellow (#EAB308) | Warning, moderate |
| Red (#EF4444) | Error, slow, failure |
| Blue (#3B82F6) | Info, interactive |
| Purple (#8B5CF6) | LLM/AI-related |
| Gray (#6B7280) | Inactive, disabled |

### Tier Colors
| Tier | Color | Badge |
|------|-------|-------|
| Tier 0 (AST) | Gray | ░░ |
| Tier 1 (Bytecode) | Blue | ██ |
| Tier 2 (JIT) | Purple | ▓▓ |

### Execution Status
| Status | Color | Animation |
|--------|-------|-----------|
| Pending | Gray | None |
| Running | Blue | Pulse |
| Success | Green | None |
| Failed | Red | None |
| Cached | Cyan | None |

---

## Implementation Recommendations

### Technology Stack
- **Framework**: React + TypeScript
- **State**: Zustand or Jotai (lightweight)
- **Visualization**: D3.js + React Flow (for DAG)
- **Charts**: Recharts or Visx
- **Animations**: Framer Motion
- **Styling**: Tailwind CSS

### Performance Optimizations
1. **Virtual scrolling** for large lists
2. **Canvas rendering** for 1000+ node graphs
3. **Web Workers** for graph layout calculations
4. **IndexedDB** for client-side caching
5. **Request coalescing** for API calls

### Accessibility
- Full keyboard navigation
- Screen reader support
- High contrast mode
- Reduced motion option
- Focus indicators

---

## Next Steps

1. **Prototype Level 2 (DAG View)** - Most complex, highest value
2. **Build component library** - Reusable across levels
3. **Implement semantic zoom** - Core differentiator
4. **Add real-time metrics streaming** - Live updates
5. **User testing** - Validate information density decisions
