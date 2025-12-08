# Product-FARM Frontend

<p align="center">
  <img src="https://img.shields.io/badge/react-19-61dafb?style=for-the-badge&logo=react" alt="React"/>
  <img src="https://img.shields.io/badge/typescript-5.0+-blue?style=for-the-badge&logo=typescript" alt="TypeScript"/>
  <img src="https://img.shields.io/badge/vite-7.2-646CFF?style=for-the-badge&logo=vite" alt="Vite"/>
  <img src="https://img.shields.io/badge/tailwindcss-4-06B6D4?style=for-the-badge&logo=tailwindcss" alt="TailwindCSS"/>
</p>

Modern React frontend for Product-FARM rule engine with visual DAG editing, real-time simulation, and AI-powered rule creation.

## Features

- **Visual Rule Builder** - Block-based drag-and-drop JSON Logic editor
- **Interactive DAG Canvas** - Rule dependency visualization with @xyflow
- **Real-time Simulation** - Test rules instantly with live feedback
- **AI Chat Assistant** - Create rules from natural language
- **Product Lifecycle Management** - Draft → Pending → Active workflow
- **Responsive Design** - Works on desktop and tablet

## Tech Stack

| Technology | Version | Purpose |
|------------|---------|---------|
| React | 19 | UI framework |
| TypeScript | 5.0+ | Type safety |
| Vite | 7.2 | Build tool |
| TailwindCSS | 4 | Styling |
| shadcn/ui | Latest | UI components |
| @xyflow/react | 12 | DAG visualization |
| Zustand | 5 | State management |
| React Router | 7 | Routing |
| Playwright | Latest | E2E testing |

## Getting Started

### Prerequisites

- Node.js 20+
- npm or yarn

### Installation

```bash
# Install dependencies
npm install

# Start development server
npm run dev
```

The frontend will be available at http://localhost:5173

### Environment Variables

Create a `.env` file (optional):

```env
# Backend API URL (defaults to http://localhost:8081)
VITE_API_URL=http://localhost:8081

# Enable mock data when backend is unavailable
VITE_USE_MOCK=false
```

## Project Structure

```
frontend/
├── src/
│   ├── components/           # React components
│   │   ├── RuleBuilder.tsx   # Block-based rule editor
│   │   ├── RuleCanvas.tsx    # DAG visualization
│   │   ├── RuleValidator.tsx # Rule validation UI
│   │   ├── SimulationPanel.tsx # Rule testing
│   │   ├── BatchEvaluator.tsx  # Batch evaluation
│   │   ├── AIChat.tsx        # AI assistant
│   │   ├── AttributeExplorer.tsx
│   │   ├── ProductCreationWizard.tsx
│   │   ├── forms/            # Form components
│   │   │   ├── ProductForm.tsx
│   │   │   ├── AbstractAttributeForm.tsx
│   │   │   ├── InlineRuleBuilder.tsx
│   │   │   └── ValueEditor.tsx
│   │   └── ui/               # shadcn/ui primitives
│   │
│   ├── pages/                # Route pages
│   │   ├── Dashboard.tsx     # Overview
│   │   ├── Products.tsx      # Product management
│   │   ├── Rules.tsx         # Rule management
│   │   ├── Attributes.tsx    # Attribute management
│   │   ├── Datatypes.tsx     # Datatype management
│   │   ├── Enumerations.tsx  # Enumeration management
│   │   ├── Functionalities.tsx
│   │   └── Settings.tsx
│   │
│   ├── services/
│   │   └── api.ts            # REST API client
│   │
│   ├── store/
│   │   └── index.ts          # Zustand state
│   │
│   ├── types/
│   │   └── index.ts          # TypeScript types
│   │
│   ├── utils/
│   │   └── validation.ts     # Client validation
│   │
│   ├── App.tsx               # Main app with routing
│   └── main.tsx              # Entry point
│
├── e2e/                      # Playwright E2E tests
│   ├── comprehensive-attributes.spec.ts
│   ├── comprehensive-datatypes.spec.ts
│   ├── comprehensive-enumerations.spec.ts
│   ├── comprehensive-rules.spec.ts
│   ├── comprehensive-simulation.spec.ts
│   ├── navigation.spec.ts
│   ├── product-workflow.spec.ts
│   ├── rules-canvas.spec.ts
│   └── simulation.spec.ts
│
├── public/                   # Static assets
├── index.html
├── package.json
├── vite.config.ts
├── tailwind.config.js
└── tsconfig.json
```

## Key Components

### RuleBuilder

Block-based visual editor for creating JSON Logic expressions:

```tsx
import { RuleBuilder } from './components/RuleBuilder';

<RuleBuilder
  productId="my-product"
  onRuleSave={(rule) => handleSave(rule)}
  availableAttributes={attributes}
/>
```

### RuleCanvas

Interactive DAG visualization using @xyflow:

```tsx
import { RuleCanvas } from './components/RuleCanvas';

<RuleCanvas
  rules={rules}
  onNodeSelect={(ruleId) => handleSelect(ruleId)}
  showExecutionLevels={true}
/>
```

### SimulationPanel

Real-time rule testing interface:

```tsx
import { SimulationPanel } from './components/SimulationPanel';

<SimulationPanel
  productId="my-product"
  onEvaluate={(inputs) => api.evaluate(productId, inputs)}
/>
```

### AIChat

AI-powered rule creation assistant:

```tsx
import { AIChat } from './components/AIChat';

<AIChat
  productId="my-product"
  onRuleGenerated={(rule) => handleNewRule(rule)}
/>
```

## Available Scripts

```bash
# Development
npm run dev           # Start dev server (port 5173)
npm run build         # Production build
npm run preview       # Preview production build

# Testing
npm run test:e2e          # Run E2E tests headless
npm run test:e2e:ui       # Run E2E tests with UI
npm run test:e2e:headed   # Run E2E tests in browser

# Code Quality
npm run lint          # Run ESLint
npm run type-check    # Run TypeScript compiler
```

## E2E Testing

The frontend includes comprehensive Playwright E2E tests:

```bash
# Install Playwright browsers (first time)
npx playwright install

# Run all E2E tests
npm run test:e2e

# Run specific test file
npx playwright test e2e/product-workflow.spec.ts

# Run with headed browser
npm run test:e2e:headed

# Open Playwright UI
npm run test:e2e:ui
```

### Test Coverage

| Test Suite | Tests | Coverage |
|------------|-------|----------|
| Navigation | 5 | Basic routing |
| Product Workflow | 8 | CRUD, lifecycle |
| Rules Canvas | 6 | DAG visualization |
| Simulation | 7 | Evaluation testing |
| Attributes | 12 | Attribute CRUD |
| Datatypes | 10 | Datatype management |
| Enumerations | 9 | Enum management |
| Rules | 11 | Rule creation |

## State Management

The app uses Zustand for global state:

```typescript
interface AppState {
  // Data
  products: Product[];
  selectedProduct: Product | null;
  rules: Rule[];
  abstractAttributes: AbstractAttribute[];

  // UI
  isLoading: boolean;
  error: string | null;
  sidebarOpen: boolean;

  // Actions
  fetchProducts: () => Promise<void>;
  createProduct: (product: CreateProductRequest) => Promise<void>;
  evaluateRules: (inputs: Record<string, any>) => Promise<EvaluationResult>;
  // ...
}
```

## API Integration

The `api.ts` service handles all backend communication:

```typescript
// Example: Evaluate rules
const result = await api.evaluate('my-product', {
  customer_age: 65,
  coverage: 250000
});

// Example: Create rule
await api.createRule('my-product', {
  rule_type: 'CALCULATION',
  expression: { '*': [{ var: 'a' }, { var: 'b' }] },
  inputs: ['a', 'b'],
  outputs: ['result']
});
```

## Styling

The app uses TailwindCSS with shadcn/ui components:

- **Theme**: Light/dark mode support
- **Colors**: Customizable via CSS variables
- **Components**: Based on Radix UI primitives
- **Responsive**: Mobile-first design

## Development Tips

### Adding a New Page

1. Create component in `src/pages/`
2. Add route in `src/App.tsx`
3. Add navigation link in sidebar

### Adding a New Component

1. Create component in `src/components/`
2. Add types in `src/types/index.ts` if needed
3. Export from component index

### Working with the Rule Builder

The rule builder uses a block-based approach:

1. Operators are organized by category (math, compare, logic)
2. Drag blocks to build expressions
3. Connect to available attributes
4. Preview JSON Logic output in real-time

## Troubleshooting

### Backend Connection Issues

```bash
# Verify backend is running
curl http://localhost:8081/api/products

# Enable mock mode in development
VITE_USE_MOCK=true npm run dev
```

### Build Errors

```bash
# Clear cache and rebuild
rm -rf node_modules/.vite
npm run build
```

### E2E Test Failures

```bash
# Update Playwright browsers
npx playwright install

# Run with debug output
DEBUG=pw:api npm run test:e2e
```

## Contributing

1. Follow the existing code style
2. Add tests for new features
3. Update types when adding new data structures
4. Run linting before committing

## License

MIT - See [LICENSE](../LICENSE) for details.
