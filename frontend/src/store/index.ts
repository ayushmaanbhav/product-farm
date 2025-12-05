import { create } from 'zustand';
import type {
  Product,
  AbstractAttribute,
  Rule,
  ProductFunctionality,
  EvaluateResponse,
  ExecutionPlan,
  AttributeValue,
  CloneProductRequest,
  CloneProductResponse,
  DataType,
} from '@/types';
import { api } from '@/services/api';

// =============================================================================
// PRODUCT STORE - Core data management
// =============================================================================

interface ProductStore {
  // State
  products: Product[];
  selectedProduct: Product | null;
  abstractAttributes: AbstractAttribute[];
  rules: Rule[];
  functionalities: ProductFunctionality[];
  executionPlan: ExecutionPlan | null;
  datatypes: DataType[];
  isLoading: boolean;
  error: string | null;

  // Product Actions
  fetchProducts: () => Promise<void>;
  selectProduct: (id: string | null) => Promise<void>;
  createProduct: (data: Partial<Product>) => Promise<Product>;
  updateProduct: (id: string, data: Partial<Product>) => Promise<void>;
  deleteProduct: (id: string) => Promise<void>;
  cloneProduct: (request: CloneProductRequest) => Promise<CloneProductResponse>;
  submitProduct: (id: string) => Promise<void>;
  approveProduct: (id: string, comments?: string) => Promise<void>;
  rejectProduct: (id: string, comments?: string) => Promise<void>;

  // Attribute Actions
  fetchAttributes: (productId: string) => Promise<void>;
  createAttribute: (data: Partial<AbstractAttribute>) => Promise<AbstractAttribute>;
  updateAttribute: (path: string, data: Partial<AbstractAttribute>) => Promise<void>;
  deleteAttribute: (path: string) => Promise<void>;

  // Rule Actions
  fetchRules: (productId: string) => Promise<void>;
  createRule: (data: Partial<Rule>) => Promise<Rule>;
  updateRule: (id: string, data: Partial<Rule>) => Promise<void>;
  deleteRule: (id: string) => Promise<void>;

  // Functionality Actions
  fetchFunctionalities: (productId: string) => Promise<void>;
  createFunctionality: (data: Partial<ProductFunctionality>) => Promise<ProductFunctionality>;
  updateFunctionality: (id: string, data: Partial<ProductFunctionality>) => Promise<void>;
  deleteFunctionality: (id: string) => Promise<void>;
  submitFunctionality: (id: string) => Promise<void>;
  approveFunctionality: (id: string, comments?: string) => Promise<void>;

  // Datatype Actions
  fetchDatatypes: () => Promise<void>;

  // Execution Plan
  fetchExecutionPlan: (productId: string) => Promise<void>;

  // Error handling
  setError: (error: string | null) => void;
  clearError: () => void;
}

export const useProductStore = create<ProductStore>((set, get) => ({
  products: [],
  selectedProduct: null,
  abstractAttributes: [],
  rules: [],
  functionalities: [],
  executionPlan: null,
  datatypes: [],
  isLoading: false,
  error: null,

  fetchProducts: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await api.products.list();
      set({ products: response.items, isLoading: false });
    } catch (e) {
      set({ error: (e as Error).message, isLoading: false });
    }
  },

  selectProduct: async (id) => {
    if (!id) {
      set({ selectedProduct: null, abstractAttributes: [], rules: [], functionalities: [], executionPlan: null });
      return;
    }
    set({ isLoading: true, error: null });
    try {
      const product = await api.products.get(id);
      const [attributes, rules, functionalities] = await Promise.all([
        api.abstractAttributes.list(id),
        api.rules.list(id),
        api.functionalities.list(id),
      ]);
      const executionPlan = await api.evaluation.getExecutionPlan(id);
      set({
        selectedProduct: product,
        abstractAttributes: attributes,
        rules,
        functionalities,
        executionPlan,
        isLoading: false,
      });
    } catch (e) {
      set({ error: (e as Error).message, isLoading: false });
    }
  },

  createProduct: async (data) => {
    set({ isLoading: true, error: null });
    try {
      const product = await api.products.create(data);
      set((state) => ({
        products: [...state.products, product],
        isLoading: false,
      }));
      return product;
    } catch (e) {
      set({ error: (e as Error).message, isLoading: false });
      throw e;
    }
  },

  updateProduct: async (id, data) => {
    set({ isLoading: true, error: null });
    try {
      const product = await api.products.update(id, data);
      set((state) => ({
        products: state.products.map((p) => (p.id === id ? product : p)),
        selectedProduct: state.selectedProduct?.id === id ? product : state.selectedProduct,
        isLoading: false,
      }));
    } catch (e) {
      set({ error: (e as Error).message, isLoading: false });
    }
  },

  deleteProduct: async (id) => {
    set({ isLoading: true, error: null });
    try {
      await api.products.delete(id);
      set((state) => ({
        products: state.products.filter((p) => p.id !== id),
        selectedProduct: state.selectedProduct?.id === id ? null : state.selectedProduct,
        isLoading: false,
      }));
    } catch (e) {
      set({ error: (e as Error).message, isLoading: false });
    }
  },

  cloneProduct: async (request) => {
    set({ isLoading: true, error: null });
    try {
      const response = await api.products.clone(request);
      // Refresh products list
      await get().fetchProducts();
      set({ isLoading: false });
      return response;
    } catch (e) {
      set({ error: (e as Error).message, isLoading: false });
      throw e;
    }
  },

  submitProduct: async (id) => {
    set({ isLoading: true, error: null });
    try {
      const product = await api.products.submit(id);
      set((state) => ({
        products: state.products.map((p) => (p.id === id ? product : p)),
        selectedProduct: state.selectedProduct?.id === id ? product : state.selectedProduct,
        isLoading: false,
      }));
    } catch (e) {
      set({ error: (e as Error).message, isLoading: false });
    }
  },

  approveProduct: async (id, comments) => {
    set({ isLoading: true, error: null });
    try {
      const product = await api.products.approve(id, comments);
      set((state) => ({
        products: state.products.map((p) => (p.id === id ? product : p)),
        selectedProduct: state.selectedProduct?.id === id ? product : state.selectedProduct,
        isLoading: false,
      }));
    } catch (e) {
      set({ error: (e as Error).message, isLoading: false });
    }
  },

  rejectProduct: async (id, comments) => {
    set({ isLoading: true, error: null });
    try {
      const product = await api.products.reject(id, comments);
      set((state) => ({
        products: state.products.map((p) => (p.id === id ? product : p)),
        selectedProduct: state.selectedProduct?.id === id ? product : state.selectedProduct,
        isLoading: false,
      }));
    } catch (e) {
      set({ error: (e as Error).message, isLoading: false });
    }
  },

  fetchAttributes: async (productId) => {
    try {
      const attributes = await api.abstractAttributes.list(productId);
      set({ abstractAttributes: attributes });
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },

  createAttribute: async (data) => {
    try {
      const attribute = await api.abstractAttributes.create(data);
      set((state) => ({
        abstractAttributes: [...state.abstractAttributes, attribute],
      }));
      return attribute;
    } catch (e) {
      set({ error: (e as Error).message });
      throw e;
    }
  },

  updateAttribute: async (path, data) => {
    try {
      const attribute = await api.abstractAttributes.update(path, data);
      set((state) => ({
        abstractAttributes: state.abstractAttributes.map((a) =>
          a.abstractPath === path ? attribute : a
        ),
      }));
    } catch (e) {
      set({ error: (e as Error).message });
      throw e;
    }
  },

  deleteAttribute: async (path) => {
    try {
      await api.abstractAttributes.delete(path);
      set((state) => ({
        abstractAttributes: state.abstractAttributes.filter((a) => a.abstractPath !== path),
      }));
    } catch (e) {
      set({ error: (e as Error).message });
      throw e;
    }
  },

  fetchRules: async (productId) => {
    try {
      const rules = await api.rules.list(productId);
      set({ rules });
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },

  createRule: async (data) => {
    try {
      const rule = await api.rules.create(data);
      set((state) => ({
        rules: [...state.rules, rule],
      }));
      // Refresh execution plan
      if (get().selectedProduct) {
        get().fetchExecutionPlan(get().selectedProduct!.id);
      }
      return rule;
    } catch (e) {
      set({ error: (e as Error).message });
      throw e;
    }
  },

  updateRule: async (id, data) => {
    try {
      const rule = await api.rules.update(id, data);
      set((state) => ({
        rules: state.rules.map((r) => (r.id === id ? rule : r)),
      }));
      // Refresh execution plan
      if (get().selectedProduct) {
        get().fetchExecutionPlan(get().selectedProduct!.id);
      }
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },

  deleteRule: async (id) => {
    try {
      await api.rules.delete(id);
      set((state) => ({
        rules: state.rules.filter((r) => r.id !== id),
      }));
      // Refresh execution plan
      if (get().selectedProduct) {
        get().fetchExecutionPlan(get().selectedProduct!.id);
      }
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },

  // Functionality Actions
  fetchFunctionalities: async (productId) => {
    try {
      const functionalities = await api.functionalities.list(productId);
      set({ functionalities });
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },

  createFunctionality: async (data) => {
    try {
      const functionality = await api.functionalities.create(data);
      set((state) => ({
        functionalities: [...state.functionalities, functionality],
      }));
      return functionality;
    } catch (e) {
      set({ error: (e as Error).message });
      throw e;
    }
  },

  updateFunctionality: async (id, data) => {
    const productId = get().selectedProduct?.id;
    if (!productId) return;
    try {
      const functionality = await api.functionalities.update(productId, id, data);
      set((state) => ({
        functionalities: state.functionalities.map((f) => (f.id === id ? functionality : f)),
      }));
    } catch (e) {
      set({ error: (e as Error).message });
      throw e;
    }
  },

  deleteFunctionality: async (id) => {
    const productId = get().selectedProduct?.id;
    if (!productId) return;
    try {
      await api.functionalities.delete(productId, id);
      set((state) => ({
        functionalities: state.functionalities.filter((f) => f.id !== id),
      }));
    } catch (e) {
      set({ error: (e as Error).message });
      throw e;
    }
  },

  submitFunctionality: async (id) => {
    const productId = get().selectedProduct?.id;
    if (!productId) return;
    try {
      const functionality = await api.functionalities.submit(productId, id);
      set((state) => ({
        functionalities: state.functionalities.map((f) => (f.id === id ? functionality : f)),
      }));
    } catch (e) {
      set({ error: (e as Error).message });
      throw e;
    }
  },

  approveFunctionality: async (id, comments) => {
    const productId = get().selectedProduct?.id;
    if (!productId) return;
    try {
      const functionality = await api.functionalities.approve(productId, id, comments);
      set((state) => ({
        functionalities: state.functionalities.map((f) => (f.id === id ? functionality : f)),
      }));
    } catch (e) {
      set({ error: (e as Error).message });
      throw e;
    }
  },

  // Datatype Actions
  fetchDatatypes: async () => {
    try {
      const datatypes = await api.datatypes.list();
      set({ datatypes });
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },

  fetchExecutionPlan: async (productId) => {
    try {
      const plan = await api.evaluation.getExecutionPlan(productId);
      set({ executionPlan: plan });
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },

  setError: (error) => set({ error }),
  clearError: () => set({ error: null }),
}));

// =============================================================================
// SIMULATION STORE - Rule testing and evaluation
// =============================================================================

interface SimulationInput {
  path: string;
  value: AttributeValue;
  displayName: string;
}

interface SimulationStore {
  inputs: SimulationInput[];
  results: EvaluateResponse | null;
  isEvaluating: boolean;
  autoEvaluate: boolean;
  scenarios: Array<{ id: string; name: string; inputs: Record<string, AttributeValue> }>;

  setInput: (path: string, value: AttributeValue) => void;
  setInputs: (inputs: SimulationInput[]) => void;
  clearInputs: () => void;
  evaluate: (productId: string) => Promise<void>;
  setAutoEvaluate: (auto: boolean) => void;
  saveScenario: (name: string) => void;
  loadScenario: (id: string) => void;
}

export const useSimulationStore = create<SimulationStore>((set, get) => ({
  inputs: [],
  results: null,
  isEvaluating: false,
  autoEvaluate: true,
  scenarios: [],

  setInput: (path, value) => {
    set((state) => {
      const existing = state.inputs.findIndex((i) => i.path === path);
      if (existing >= 0) {
        const newInputs = [...state.inputs];
        newInputs[existing] = { ...newInputs[existing], value };
        return { inputs: newInputs };
      }
      return { inputs: [...state.inputs, { path, value, displayName: path }] };
    });
  },

  setInputs: (inputs) => set({ inputs }),
  clearInputs: () => set({ inputs: [], results: null }),

  evaluate: async (productId) => {
    const { inputs } = get();
    if (inputs.length === 0) return;

    set({ isEvaluating: true });
    try {
      const inputData: Record<string, AttributeValue> = {};
      inputs.forEach((i) => {
        const shortPath = i.path.split(':').pop() || i.path;
        inputData[shortPath] = i.value;
      });

      const results = await api.evaluation.evaluate({
        productId,
        inputData,
      });
      set({ results, isEvaluating: false });
    } catch (e) {
      set({ isEvaluating: false });
    }
  },

  setAutoEvaluate: (auto) => set({ autoEvaluate: auto }),

  saveScenario: (name) => {
    const { inputs, scenarios } = get();
    const inputData: Record<string, AttributeValue> = {};
    inputs.forEach((i) => {
      inputData[i.path] = i.value;
    });
    set({
      scenarios: [...scenarios, { id: crypto.randomUUID(), name, inputs: inputData }],
    });
  },

  loadScenario: (id) => {
    const scenario = get().scenarios.find((s) => s.id === id);
    if (scenario) {
      const inputs: SimulationInput[] = Object.entries(scenario.inputs).map(([path, value]) => ({
        path,
        value,
        displayName: path.split(':').pop() || path,
      }));
      set({ inputs });
    }
  },
}));

// =============================================================================
// UI STORE - Global UI state
// =============================================================================

type ViewMode = 'graph' | 'table' | 'split';
type GraphLayout = 'dagre' | 'hierarchical' | 'force';
type FilterType = 'all' | 'functionality' | 'component' | 'tag';

interface UIStore {
  sidebarOpen: boolean;
  chatOpen: boolean;
  simulationPanelOpen: boolean;
  attributeExplorerOpen: boolean;
  functionalityPanelOpen: boolean;
  cloneDialogOpen: boolean;
  theme: 'light' | 'dark';
  viewMode: ViewMode;
  graphLayout: GraphLayout;
  selectedNodeId: string | null;
  highlightedRuleIds: string[];
  impactAnalysisTarget: string | null;
  filterType: FilterType;
  selectedFunctionalityId: string | null;
  selectedComponentType: string | null;
  selectedTag: string | null;

  toggleSidebar: () => void;
  toggleChat: () => void;
  toggleSimulationPanel: () => void;
  toggleAttributeExplorer: () => void;
  toggleFunctionalityPanel: () => void;
  setCloneDialogOpen: (open: boolean) => void;
  setTheme: (theme: 'light' | 'dark') => void;
  setViewMode: (mode: ViewMode) => void;
  setGraphLayout: (layout: GraphLayout) => void;
  selectNode: (id: string | null) => void;
  highlightRules: (ruleIds: string[]) => void;
  setImpactAnalysisTarget: (path: string | null) => void;
  setFilterType: (type: FilterType) => void;
  setSelectedFunctionality: (id: string | null) => void;
  setSelectedComponent: (type: string | null) => void;
  setSelectedTag: (tag: string | null) => void;
  clearFilters: () => void;
}

export const useUIStore = create<UIStore>((set) => ({
  sidebarOpen: true,
  chatOpen: false,
  simulationPanelOpen: true,
  attributeExplorerOpen: true,
  functionalityPanelOpen: false,
  cloneDialogOpen: false,
  theme: 'light',
  viewMode: 'split',
  graphLayout: 'dagre',
  selectedNodeId: null,
  highlightedRuleIds: [],
  impactAnalysisTarget: null,
  filterType: 'all',
  selectedFunctionalityId: null,
  selectedComponentType: null,
  selectedTag: null,

  toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
  toggleChat: () => set((state) => ({ chatOpen: !state.chatOpen })),
  toggleSimulationPanel: () => set((state) => ({ simulationPanelOpen: !state.simulationPanelOpen })),
  toggleAttributeExplorer: () => set((state) => ({ attributeExplorerOpen: !state.attributeExplorerOpen })),
  toggleFunctionalityPanel: () => set((state) => ({ functionalityPanelOpen: !state.functionalityPanelOpen })),
  setCloneDialogOpen: (open) => set({ cloneDialogOpen: open }),
  setTheme: (theme) => set({ theme }),
  setViewMode: (mode) => set({ viewMode: mode }),
  setGraphLayout: (layout) => set({ graphLayout: layout }),
  selectNode: (id) => set({ selectedNodeId: id }),
  highlightRules: (ruleIds) => set({ highlightedRuleIds: ruleIds }),
  setImpactAnalysisTarget: (path) => set({ impactAnalysisTarget: path }),
  setFilterType: (type) => set({ filterType: type }),
  setSelectedFunctionality: (id) => set({ selectedFunctionalityId: id, filterType: id ? 'functionality' : 'all' }),
  setSelectedComponent: (type) => set({ selectedComponentType: type, filterType: type ? 'component' : 'all' }),
  setSelectedTag: (tag) => set({ selectedTag: tag, filterType: tag ? 'tag' : 'all' }),
  clearFilters: () => set({
    filterType: 'all',
    selectedFunctionalityId: null,
    selectedComponentType: null,
    selectedTag: null,
  }),
}));
