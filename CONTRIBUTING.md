# Contributing to Product-FARM

First off, thank you for considering contributing to Product-FARM! It's people like you that make this project great.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please be respectful, inclusive, and considerate in all interactions.

## Getting Started

### Prerequisites

- **Rust** 1.75+ ([rustup.rs](https://rustup.rs))
- **Node.js** 20+ ([nodejs.org](https://nodejs.org))
- **Git** 2.0+

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/product-farm.git
   cd product-farm
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/ORIGINAL_OWNER/product-farm.git
   ```

## Development Setup

### Backend (Rust)

```bash
cd backend

# Build
cargo build

# Run tests
cargo test --workspace

# Run with logging
RUST_LOG=debug cargo run -p product-farm-api

# Run benchmarks
cargo bench --workspace

# Check formatting
cargo fmt --check

# Run clippy lints
cargo clippy --workspace -- -D warnings
```

### Frontend (React/TypeScript)

```bash
cd frontend

# Install dependencies
npm install

# Start dev server
npm run dev

# Run E2E tests
npm run test:e2e

# Lint
npm run lint

# Type check
npm run type-check
```

### Full Stack

```bash
# Start all services
./start-all.sh

# Stop all services
./stop-all.sh
```

## How to Contribute

### Reporting Bugs

Before creating a bug report:
1. Check existing issues to avoid duplicates
2. Collect relevant information:
   - OS and version
   - Rust/Node.js versions
   - Steps to reproduce
   - Expected vs actual behavior
   - Error messages and logs

Create a bug report with:
- Clear, descriptive title
- Detailed description with steps to reproduce
- Code samples if applicable
- Screenshots for UI issues

### Suggesting Features

Feature requests are welcome! Please:
1. Check existing issues for similar requests
2. Describe the problem you're trying to solve
3. Explain your proposed solution
4. Consider alternative approaches

### Contributing Code

1. **Find an issue** - Look for issues labeled `good first issue` or `help wanted`
2. **Comment on the issue** - Let others know you're working on it
3. **Create a branch** - Use descriptive branch names:
   - `feature/add-rule-validation`
   - `fix/cycle-detection-bug`
   - `docs/update-api-reference`
4. **Make your changes** - Follow the coding standards
5. **Write tests** - Ensure adequate test coverage
6. **Submit a PR** - Reference the related issue

## Pull Request Process

### Before Submitting

1. **Rebase on latest main**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all tests**:
   ```bash
   # Backend
   cd backend && cargo test --workspace

   # Frontend
   cd frontend && npm run test:e2e
   ```

3. **Check formatting and lints**:
   ```bash
   # Rust
   cargo fmt --check
   cargo clippy --workspace -- -D warnings

   # TypeScript
   npm run lint
   npm run type-check
   ```

4. **Update documentation** if needed

### PR Guidelines

- **Title**: Use conventional commit format
  - `feat: add batch evaluation API`
  - `fix: resolve cycle detection false positive`
  - `docs: update quick start guide`
  - `refactor: simplify DAG builder`
  - `test: add JSON Logic edge cases`

- **Description**: Include:
  - What changes were made
  - Why the changes were necessary
  - How to test the changes
  - Screenshots for UI changes

- **Size**: Keep PRs focused and reasonably sized
  - Large changes should be split into smaller PRs
  - Each PR should do one thing well

### Review Process

1. Automated checks must pass (CI/CD)
2. At least one maintainer approval required
3. All review comments must be addressed
4. Branch must be up-to-date with main

## Coding Standards

### Rust

- Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `rustfmt` for formatting
- Address all `clippy` warnings
- Write documentation for public APIs
- Use meaningful variable and function names
- Prefer explicit error handling over `unwrap()`

```rust
// Good
pub fn evaluate_rule(rule: &Rule, context: &Context) -> Result<Value, EvalError> {
    let expression = rule.get_expression()?;
    evaluator.evaluate(&expression, context)
}

// Avoid
pub fn eval(r: &Rule, c: &Context) -> Result<Value, EvalError> {
    evaluator.evaluate(&r.get_expression().unwrap(), c)
}
```

### TypeScript/React

- Follow the ESLint configuration
- Use TypeScript strict mode
- Prefer functional components with hooks
- Use meaningful component and variable names
- Add JSDoc comments for complex functions

```typescript
// Good
interface RuleEditorProps {
  rule: Rule;
  onSave: (rule: Rule) => Promise<void>;
  disabled?: boolean;
}

export function RuleEditor({ rule, onSave, disabled = false }: RuleEditorProps) {
  // ...
}

// Avoid
export function RE({ r, cb, d }: any) {
  // ...
}
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting (no code change)
- `refactor`: Code change (no new feature or bug fix)
- `perf`: Performance improvement
- `test`: Adding/updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(rule-engine): add parallel execution for independent rules

Implements parallel execution for rules in the same DAG level.
Rules without dependencies now execute concurrently using tokio.

Closes #123
```

## Testing Guidelines

### Backend Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_evaluation_simple() {
        // Arrange
        let rule = Rule::new("test", "CALC", json!({"*": [{"var": "x"}, 2]}));
        let context = Context::new().with("x", 21);

        // Act
        let result = evaluate(&rule, &context).unwrap();

        // Assert
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_rule_evaluation_error() {
        // Test error cases
    }
}
```

### Frontend Tests (Playwright)

```typescript
test.describe('Rule Builder', () => {
  test('should create a simple calculation rule', async ({ page }) => {
    // Arrange
    await page.goto('/rules');

    // Act
    await page.click('button:has-text("New Rule")');
    await page.fill('[data-testid="rule-name"]', 'Test Rule');

    // Assert
    await expect(page.locator('[data-testid="rule-preview"]')).toBeVisible();
  });
});
```

### Test Coverage

- Aim for 80%+ code coverage
- Focus on critical paths and edge cases
- Don't write tests just for coverage numbers
- Test behavior, not implementation details

## Documentation

### Code Documentation

**Rust:**
```rust
/// Evaluates a JSON Logic expression against the given context.
///
/// # Arguments
///
/// * `expression` - The JSON Logic expression to evaluate
/// * `context` - The variable context for evaluation
///
/// # Returns
///
/// The result of evaluation, or an error if evaluation fails.
///
/// # Examples
///
/// ```
/// let expr = json!({"*": [{"var": "x"}, 2]});
/// let ctx = Context::new().with("x", 21);
/// let result = evaluate(&expr, &ctx)?;
/// assert_eq!(result, Value::Int(42));
/// ```
pub fn evaluate(expression: &Expression, context: &Context) -> Result<Value, EvalError> {
    // ...
}
```

**TypeScript:**
```typescript
/**
 * Evaluates rules for a product with the given inputs.
 *
 * @param productId - The ID of the product to evaluate
 * @param inputs - Input values for rule evaluation
 * @returns Promise resolving to evaluation results
 * @throws ApiError if the request fails
 *
 * @example
 * const result = await api.evaluate('my-product', { age: 65, coverage: 250000 });
 * console.log(result.outputs.premium);
 */
export async function evaluate(
  productId: string,
  inputs: Record<string, unknown>
): Promise<EvaluationResult> {
  // ...
}
```

### README Updates

When adding features:
1. Update the main README.md if it's a major feature
2. Add examples to relevant documentation
3. Update API reference if endpoints change

## Questions?

- Open a [Discussion](https://github.com/OWNER/product-farm/discussions) for questions
- Join our community chat (if available)
- Check existing issues and documentation

Thank you for contributing to Product-FARM!
