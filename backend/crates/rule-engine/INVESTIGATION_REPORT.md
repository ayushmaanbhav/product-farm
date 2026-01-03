# Rule Engine Investigation Report

**Date**: 2026-01-03
**Scope**: Full rule-engine ecosystem analysis
**Status**: Initial investigation complete

---

## Critical Issues (P0 - Fix Immediately)

### 1. Thread Safety: RuleExecutor & ProductRegistry NOT Thread-Safe
- **Location**: `rule-engine/src/executor.rs:82-87`, `yaml-loader/src/registry.rs:17-26`
- **Problem**: Both use plain `HashMap` without Mutex/RwLock. Concurrent calls will cause race conditions or panics.
- **Impact**: Production crash in multi-threaded environments
- **Status**: ✅ FIXED - RuleExecutor now uses Arc<HashMap> for compiled rules, execute() takes &self

### 2. Chain Comparison Bytecode Bug
- **Location**: `json-logic/src/compiler.rs:459-481`
- **Problem**: Expression `a < b < c` compiles to semantically incorrect bytecode. The `And` opcode is emitted in wrong position.
- **Impact**: Wrong evaluation results for chained comparisons
- **Status**: ✅ VERIFIED CORRECT - Bytecode tests confirm chain comparison works correctly. Added test_vm_chain_comparison.

### 3. Silent Error Suppression in Parallel Execution
- **Location**: `rule-engine/src/executor.rs:217-220`
- **Problem**: Only the FIRST error from parallel execution is reported. All others are discarded.
- **Impact**: Extremely difficult to debug parallel execution failures
- **Status**: ✅ FIXED - Added MultipleRuleFailures error variant that aggregates all errors with rule IDs

### 4. Retries Parse Errors (Wastes Resources)
- **Location**: `llm-evaluator/src/executor.rs:310-398`
- **Problem**: Retries ALL failures including parse errors and config errors that won't be fixed by retry.
- **Impact**: Wasted API calls and delayed error reporting
- **Status**: ✅ FIXED - Added is_retryable() to LlmEvaluatorError, only retries network/rate-limit/server/timeout errors

---

## High Priority Issues (P1)

### 5. Missing Dependency Validation
- **Location**: `rule-engine/src/dag.rs:256-270`
- **Problem**: `find_missing_inputs()` exists but is NEVER called. Rules with unsatisfiable dependencies fail at runtime.
- **Impact**: Silent failures when dependencies aren't met
- **Status**: OPEN

### 6. Circular Dependency Detection Missing
- **Location**: `yaml-loader/src/transformer.rs:414-432`
- **Problem**: `CircularDependency` error type exists but is never raised. No validation for circular function dependencies.
- **Impact**: Potential infinite loops during evaluation
- **Status**: OPEN

### 7. Silent LLM Fallback to NoOp
- **Location**: `yaml-loader/src/registry.rs:76-110`
- **Problem**: If Claude/Ollama initialization fails, silently uses NoOpLlmEvaluator. User doesn't know until runtime.
- **Impact**: Hidden failures, confusing debugging
- **Status**: OPEN

### 8. Array Operations NOT Bytecode-Compiled
- **Location**: `json-logic/src/compiler.rs:388-401`
- **Problem**: `map`, `filter`, `reduce`, `all`, `some`, `none`, `merge` explicitly rejected from bytecode, always use slow AST path.
- **Impact**: Performance degradation for array-heavy rules
- **Status**: OPEN

### 9. Sub Operation Ignores Extra Arguments
- **Location**: `json-logic/src/compiler.rs:338-344`
- **Problem**: `{"-": [1,2,3,4]}` only computes `1-2`, silently ignores `3,4`.
- **Impact**: Silent data loss in arithmetic
- **Status**: OPEN

---

## Medium Priority Issues (P2)

### 10. Hardcoded Configuration Values

| Value | Location | Default | Should Be Configurable |
|-------|----------|---------|----------------------|
| `PROMOTION_THRESHOLD` | `json-logic/tiered.rs:20` | 100 | Yes - tiered compilation threshold |
| `BYTECODE_COMPILE_THRESHOLD` | `json-logic/evaluator.rs:18` | 5 nodes | Yes - compilation decision |
| `MAX_STACK_DEPTH` | `json-logic/vm.rs:12` | 1024 | Yes - stack overflow protection |
| Confidence threshold | `yaml-loader/transformer.rs:651` | 0.5 | Yes - low-confidence flagging |
| API version | `llm-evaluator/env_config.rs:207` | "2023-06-01" | Yes - outdated, should update |

### 11. Double HashMap Conversion Every Evaluation
- **Location**: `yaml-loader/src/registry.rs:282-287`
- **Problem**: Converts `std::HashMap` → `hashbrown::HashMap` on every evaluation call.
- **Impact**: Unnecessary allocations and clones
- **Status**: OPEN

### 12. Naive Number/Boolean Parsing in LLM Responses
- **Location**: `llm-evaluator/claude.rs:258-273`, `ollama.rs:229-245`
- **Problem**: Number parsing via character filtering: `"text-123-456"` → `123456` (wrong)
- **Impact**: Incorrect parsed values from LLM responses
- **Status**: OPEN

### 13. No Timeout/Cancellation Mechanism
- **Location**: `rule-engine/src/executor.rs`
- **Problem**: No way to interrupt long-running rule evaluations. No configurable timeout.
- **Impact**: Hang-prone in production
- **Status**: OPEN

### 14. LLM Error Details Lost
- **Location**: `yaml-loader/src/registry.rs:298`
- **Problem**: `LlmEvaluatorError` not wrapped by `LoaderError`. Original API error lost.
- **Impact**: Generic error messages, difficult debugging
- **Status**: OPEN

### 15. Context Cloned Per Parallel Level
- **Location**: `rule-engine/src/executor.rs:198`
- **Problem**: Entire context cloned and converted to JSON for each parallel level.
- **Impact**: Performance degradation with large contexts
- **Status**: OPEN

---

## Low Priority / Code Quality (P3)

### 16. Inconsistent Naming Across Crates
| Concept | Core | Rule-Engine | JSON-Logic | YAML-Loader |
|---------|------|-------------|------------|-------------|
| Execution state | - | ExecutionContext | EvalContext | State |
| Error type | CoreError | RuleEngineError | JsonLogicError | LoaderError |
| Output | - | RuleResult | - | EvalResult |

### 17. Missing Integration Tests
- No FarmScript → JSON Logic → Execution pipeline tests
- No mixed evaluator type tests (JsonLogic + LLM in same DAG)
- No feature flag integration tests
- No error path integration tests

### 18. Excessive `.unwrap()` in Tests
- **Locations**: Multiple test files in `claude.rs`, `ollama.rs`
- **Problem**: Tests use `.unwrap()` instead of `.expect()` with context
- **Impact**: Unhelpful panic messages on test failures

### 19. No Iteration Limits in Array Operations
- **Location**: `json-logic/evaluator.rs:348-396`
- **Problem**: `map`, `filter`, `reduce` have no iteration limits.
- **Impact**: Potential DoS with large arrays + expensive operations

### 20. Path Format Is Fragile Contract
- **Location**: `core/src/types.rs:220-251`
- **Problem**: Path parsing depends on hardcoded "abstract-path" marker. Any format change breaks all parsing.
- **Impact**: High breaking change risk

---

## Hardcoded Values Requiring Externalization

| Category | Value | Location | Recommendation |
|----------|-------|----------|----------------|
| **Compilation** | Promotion threshold: 100 | `json-logic/tiered.rs:20` | Add to config |
| **Compilation** | Bytecode threshold: 5 nodes | `json-logic/evaluator.rs:18` | Add to config |
| **VM Safety** | Max stack depth: 1024 | `json-logic/vm.rs:12` | Add to config |
| **Cloning** | Max attributes: 100,000 | `core/clone.rs:18-27` | Environment variable |
| **LLM** | Anthropic concurrency: 5 | `llm-evaluator/env_config.rs` | ✓ Already configurable |
| **LLM** | Ollama concurrency: 10 | `llm-evaluator/env_config.rs` | ✓ Already configurable |
| **Validation** | Confidence threshold: 0.5 | `yaml-loader/transformer.rs:651` | Add to config |
| **API** | Claude API version: 2023-06-01 | `llm-evaluator/claude.rs:89` | Update + make configurable |
| **Defaults** | Template type: "generic" | `yaml-loader/transformer.rs:203` | Add to config |
| **Defaults** | Display expression version: "1.0" | `core/rule.rs:239` | Document or configure |

---

## Action Plan

### Immediate (P0 - Before Production)
1. [ ] Wrap `RuleExecutor` and `ProductRegistry` in `Arc<RwLock<>>` for thread safety
2. [ ] Fix chain comparison bytecode generation bug
3. [ ] Aggregate all errors in parallel execution (not just first)
4. [ ] Only retry transient errors (timeout, 5xx), not parse/config errors

### Short-term (P1 - Next Sprint)
5. [ ] Add `find_missing_inputs()` call during rule compilation
6. [ ] Implement circular dependency detection in yaml-loader
7. [ ] Make tiered compilation thresholds configurable
8. [ ] Fix number parsing to use regex/proper parsing

### Medium-term (P2)
9. [ ] Implement array operations in bytecode compiler
10. [ ] Add timeout mechanism for rule execution
11. [ ] Standardize naming (ExecutionContext everywhere)
12. [ ] Add comprehensive integration tests
