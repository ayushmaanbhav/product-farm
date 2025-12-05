// API Service Layer for Product-FARM
// Connects to gRPC backend via REST gateway or grpc-web

import type {
  Product,
  AbstractAttribute,
  Attribute,
  Rule,
  ProductFunctionality,
  FunctionalityRequiredAttribute,
  DataType,
  TemplateEnumeration,
  ProductTemplate,
  EvaluateRequest,
  EvaluateResponse,
  ExecutionPlan,
  ValidationResult,
  AttributeValue,
  PaginatedResponse,
  CloneProductRequest,
  CloneProductResponse,
  EvaluateFunctionalityRequest,
  EvaluateFunctionalityResponse,
  ImpactAnalysis,
  DependencyInfo,
  BatchEvaluateRequest,
  BatchEvaluateResponse,
} from '@/types';

// =============================================================================
// CONFIGURATION
// =============================================================================

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const USE_MOCK = import.meta.env.VITE_USE_MOCK === 'true'; // Set VITE_USE_MOCK=true to use mock data

// =============================================================================
// HTTP UTILITIES
// =============================================================================

async function fetchJson<T>(
  url: string,
  options?: RequestInit
): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${url}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || `HTTP ${response.status}`);
  }

  return response.json();
}

// =============================================================================
// MOCK DATA STORE
// =============================================================================

let mockProducts: Product[] = [];
let mockAbstractAttributes: AbstractAttribute[] = [];
let mockAttributes: Attribute[] = [];
let mockRules: Rule[] = [];
let mockFunctionalities: ProductFunctionality[] = [];
let mockDatatypes: DataType[] = [];
let mockTemplateEnumerations: TemplateEnumeration[] = [];
let mockProductTemplates: ProductTemplate[] = [];

function generateId(): string {
  return crypto.randomUUID().replace(/-/g, '').substring(0, 32);
}

function now(): number {
  return Math.floor(Date.now() / 1000);
}

// Initialize with sample data
function initMockData() {
  if (mockProducts.length > 0) return;

  // Sample datatypes
  mockDatatypes = [
    { id: 'integer', name: 'Integer', primitiveType: 'INT', constraints: { min: -2147483648, max: 2147483647 }, description: 'Whole number' },
    { id: 'decimal', name: 'Decimal', primitiveType: 'DECIMAL', constraints: { precision: 10, scale: 2 }, description: 'Decimal number' },
    { id: 'string', name: 'String', primitiveType: 'STRING', constraints: { maxLength: 255 }, description: 'Text value' },
    { id: 'boolean', name: 'Boolean', primitiveType: 'BOOL', constraints: {}, description: 'True/False value' },
    { id: 'enum', name: 'Enumeration', primitiveType: 'ENUM', constraints: {}, description: 'Choice from predefined values' },
    { id: 'date', name: 'Date', primitiveType: 'DATETIME', constraints: {}, description: 'Date value' },
    { id: 'percentage', name: 'Percentage', primitiveType: 'FLOAT', constraints: { min: 0, max: 100 }, description: 'Percentage value' },
  ];

  // Sample template enumerations
  mockTemplateEnumerations = [
    { id: 'cover_types', name: 'Cover Types', templateType: 'insurance', values: ['comprehensive', 'third_party', 'third_party_fire_theft'], description: 'Insurance cover types' },
    { id: 'vehicle_types', name: 'Vehicle Types', templateType: 'insurance', values: ['car', 'motorcycle', 'truck', 'van'], description: 'Vehicle categories' },
    { id: 'payment_freq', name: 'Payment Frequency', templateType: 'insurance', values: ['monthly', 'quarterly', 'annually'], description: 'Payment frequency options' },
  ];

  // Sample product
  const productId = 'insurance_motor_v1';
  mockProducts.push({
    id: productId,
    name: 'Motor Insurance Product',
    description: 'Comprehensive motor insurance with multiple covers',
    templateType: 'insurance',
    status: 'DRAFT',
    effectiveFrom: now(),
    createdAt: now(),
    updatedAt: now(),
    version: 1,
  });

  // Sample functionalities
  mockFunctionalities = [
    {
      id: 'func_premium',
      productId,
      name: 'premium_calculation',
      displayName: 'Premium Calculation',
      description: 'Core premium calculation functionality',
      status: 'ACTIVE',
      immutable: false,
      requiredAttributes: [
        { functionalityId: 'func_premium', abstractPath: `${productId}:abstract-path:customer:customer_age`, description: 'Customer age for rating', orderIndex: 0 },
        { functionalityId: 'func_premium', abstractPath: `${productId}:abstract-path:vehicle:vehicle_value`, description: 'Vehicle value for base premium', orderIndex: 1 },
        { functionalityId: 'func_premium', abstractPath: `${productId}:abstract-path:pricing:final_premium`, description: 'Final premium output', orderIndex: 2 },
      ],
      createdAt: now() - 86400,
      updatedAt: now(),
    },
    {
      id: 'func_discount',
      productId,
      name: 'discount_rules',
      displayName: 'Discount Rules',
      description: 'Discount calculation based on cover type and loyalty',
      status: 'DRAFT',
      immutable: false,
      requiredAttributes: [
        { functionalityId: 'func_discount', abstractPath: `${productId}:abstract-path:cover:cover_type`, description: 'Cover type for discount', orderIndex: 0 },
        { functionalityId: 'func_discount', abstractPath: `${productId}:abstract-path:pricing:discount_rate`, description: 'Discount rate output', orderIndex: 1 },
      ],
      createdAt: now() - 43200,
      updatedAt: now(),
    },
    {
      id: 'func_eligibility',
      productId,
      name: 'eligibility_check',
      displayName: 'Eligibility Check',
      description: 'Customer eligibility validation',
      status: 'ACTIVE',
      immutable: true, // This functionality is immutable
      requiredAttributes: [
        { functionalityId: 'func_eligibility', abstractPath: `${productId}:abstract-path:customer:customer_age`, description: 'Age for eligibility', orderIndex: 0 },
      ],
      createdAt: now() - 172800,
      updatedAt: now(),
    },
  ];

  // Sample abstract attributes
  const attributes = [
    { name: 'customer_age', component: 'customer', datatype: 'integer', isInput: true, tags: ['input', 'customer'], immutable: false },
    { name: 'vehicle_value', component: 'vehicle', datatype: 'decimal', isInput: true, tags: ['input', 'vehicle'], immutable: false },
    { name: 'vehicle_age', component: 'vehicle', datatype: 'integer', isInput: true, tags: ['input', 'vehicle'], immutable: false },
    { name: 'cover_type', component: 'cover', datatype: 'enum', isInput: true, tags: ['input', 'cover'], immutable: false, enumName: 'cover_types' },
    { name: 'base_premium', component: 'pricing', datatype: 'decimal', isInput: false, tags: ['output', 'premium'], immutable: true }, // Immutable
    { name: 'age_factor', component: 'pricing', datatype: 'decimal', isInput: false, tags: ['output', 'factor'], immutable: false },
    { name: 'vehicle_factor', component: 'pricing', datatype: 'decimal', isInput: false, tags: ['output', 'factor'], immutable: false },
    { name: 'total_premium', component: 'pricing', datatype: 'decimal', isInput: false, tags: ['output', 'premium'], immutable: false },
    { name: 'discount_rate', component: 'pricing', datatype: 'decimal', isInput: false, tags: ['output', 'discount'], immutable: false },
    { name: 'final_premium', component: 'pricing', datatype: 'decimal', isInput: false, tags: ['output', 'premium'], immutable: true }, // Immutable
    { name: 'is_eligible', component: 'eligibility', datatype: 'boolean', isInput: false, tags: ['output', 'eligibility'], immutable: true }, // Immutable
  ];

  attributes.forEach((attr) => {
    const abstractPath = `${productId}:abstract-path:${attr.component}:${attr.name}`;
    const humanReadableName = attr.name.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
    mockAbstractAttributes.push({
      abstractPath,
      productId,
      componentType: attr.component,
      attributeName: attr.name,
      datatypeId: attr.datatype,
      enumName: attr.enumName,
      description: `${humanReadableName} attribute`,
      displayExpression: attr.name.replace(/_/g, ' '),
      displayNames: [
        { name: attr.name, format: 'SYSTEM', orderIndex: 0 },
        { name: humanReadableName, format: 'HUMAN', orderIndex: 1 },
      ],
      tags: attr.tags.map((t, i) => ({ name: t, orderIndex: i })),
      relatedAttributes: [],
      immutable: attr.immutable,
      createdAt: now(),
      updatedAt: now(),
    });

    // Create corresponding attribute instances
    const componentId = 'default';
    mockAttributes.push({
      path: `${productId}:${attr.component}:${componentId}:${attr.name}`,
      abstractPath,
      productId,
      componentType: attr.component,
      componentId,
      attributeName: attr.name,
      valueType: attr.isInput ? 'JUST_DEFINITION' : 'RULE_DRIVEN',
      value: attr.isInput ? undefined : { type: 'null' },
      ruleId: attr.isInput ? undefined : undefined,
      createdAt: now(),
      updatedAt: now(),
    });
  });

  // Sample rules
  const rules = [
    {
      ruleType: 'calculation',
      displayExpression: 'base_premium = vehicle_value * 0.05',
      expression: { '*': [{ 'var': 'vehicle_value' }, 0.05] },
      inputs: ['vehicle:vehicle_value'],
      outputs: ['pricing:base_premium'],
      description: 'Calculate base premium as 5% of vehicle value',
    },
    {
      ruleType: 'calculation',
      displayExpression: 'age_factor = IF customer_age < 25 THEN 1.5 ELSE IF customer_age > 65 THEN 1.3 ELSE 1.0',
      expression: { 'if': [[{ '<': [{ 'var': 'customer_age' }, 25] }, 1.5, { '>': [{ 'var': 'customer_age' }, 65] }, 1.3, 1.0]] },
      inputs: ['customer:customer_age'],
      outputs: ['pricing:age_factor'],
      description: 'Age-based premium factor',
    },
    {
      ruleType: 'calculation',
      displayExpression: 'vehicle_factor = IF vehicle_age > 10 THEN 0.7 ELSE IF vehicle_age > 5 THEN 0.85 ELSE 1.0',
      expression: { 'if': [[{ '>': [{ 'var': 'vehicle_age' }, 10] }, 0.7, { '>': [{ 'var': 'vehicle_age' }, 5] }, 0.85, 1.0]] },
      inputs: ['vehicle:vehicle_age'],
      outputs: ['pricing:vehicle_factor'],
      description: 'Vehicle age depreciation factor',
    },
    {
      ruleType: 'calculation',
      displayExpression: 'total_premium = base_premium * age_factor * vehicle_factor',
      expression: { '*': [{ '*': [{ 'var': 'base_premium' }, { 'var': 'age_factor' }] }, { 'var': 'vehicle_factor' }] },
      inputs: ['pricing:base_premium', 'pricing:age_factor', 'pricing:vehicle_factor'],
      outputs: ['pricing:total_premium'],
      description: 'Calculate total premium with all factors',
    },
    {
      ruleType: 'calculation',
      displayExpression: 'discount_rate = IF cover_type == "comprehensive" THEN 0.1 ELSE 0.05',
      expression: { 'if': [[{ '==': [{ 'var': 'cover_type' }, 'comprehensive'] }, 0.1, 0.05]] },
      inputs: ['cover:cover_type'],
      outputs: ['pricing:discount_rate'],
      description: 'Cover type discount',
    },
    {
      ruleType: 'calculation',
      displayExpression: 'final_premium = total_premium * (1 - discount_rate)',
      expression: { '*': [{ 'var': 'total_premium' }, { '-': [1, { 'var': 'discount_rate' }] }] },
      inputs: ['pricing:total_premium', 'pricing:discount_rate'],
      outputs: ['pricing:final_premium'],
      description: 'Calculate final premium after discount',
    },
    {
      ruleType: 'validation',
      displayExpression: 'is_eligible = customer_age >= 18 AND customer_age <= 85',
      expression: { 'and': [{ '>=': [{ 'var': 'customer_age' }, 18] }, { '<=': [{ 'var': 'customer_age' }, 85] }] },
      inputs: ['customer:customer_age'],
      outputs: ['eligibility:is_eligible'],
      description: 'Check customer age eligibility',
    },
  ];

  rules.forEach((rule, idx) => {
    const ruleId = generateId();
    mockRules.push({
      id: ruleId,
      productId,
      ruleType: rule.ruleType,
      displayExpression: rule.displayExpression,
      compiledExpression: JSON.stringify(rule.expression),
      description: rule.description,
      enabled: true,
      orderIndex: idx,
      inputAttributes: rule.inputs.map((path, i) => ({
        ruleId,
        attributePath: `${productId}:abstract-path:${path}`,
        orderIndex: i,
      })),
      outputAttributes: rule.outputs.map((path, i) => ({
        ruleId,
        attributePath: `${productId}:abstract-path:${path}`,
        orderIndex: i,
      })),
      createdAt: now(),
      updatedAt: now(),
    });
  });

  // Initialize Product Templates
  mockProductTemplates = [
    {
      id: 'tpl_motor_insurance',
      name: 'Motor Insurance',
      description: 'Comprehensive motor insurance product template with premium calculation, discounts, and eligibility rules',
      type: 'insurance',
      version: '1.0.0',
      enumerations: [
        { id: 'cover_types', name: 'Cover Types', templateType: 'insurance', values: ['comprehensive', 'third_party', 'third_party_fire_theft'], description: 'Insurance cover types' },
        { id: 'vehicle_types', name: 'Vehicle Types', templateType: 'insurance', values: ['car', 'motorcycle', 'truck', 'van'], description: 'Vehicle categories' },
        { id: 'payment_freq', name: 'Payment Frequency', templateType: 'insurance', values: ['monthly', 'quarterly', 'annually'], description: 'Payment frequency options' },
      ],
      components: [
        { id: 'customer', name: 'customer', displayName: 'Customer', description: 'Customer information and demographics', isRequired: true, orderIndex: 0 },
        { id: 'vehicle', name: 'vehicle', displayName: 'Vehicle', description: 'Vehicle details and specifications', isRequired: true, orderIndex: 1 },
        { id: 'cover', name: 'cover', displayName: 'Cover', description: 'Coverage options and selections', isRequired: true, orderIndex: 2 },
        { id: 'pricing', name: 'pricing', displayName: 'Pricing', description: 'Premium calculation and discounts', isRequired: true, orderIndex: 3 },
        { id: 'eligibility', name: 'eligibility', displayName: 'Eligibility', description: 'Eligibility rules and validation', isRequired: false, orderIndex: 4 },
        { id: 'claims', name: 'claims', displayName: 'Claims', description: 'Claims processing attributes', isRequired: false, orderIndex: 5 },
      ],
      datatypes: [
        { datatypeId: 'integer', isRequired: true, defaultIncluded: true },
        { datatypeId: 'decimal', isRequired: true, defaultIncluded: true },
        { datatypeId: 'string', isRequired: true, defaultIncluded: true },
        { datatypeId: 'boolean', isRequired: true, defaultIncluded: true },
        { datatypeId: 'enum', isRequired: true, defaultIncluded: true },
        { datatypeId: 'date', isRequired: false, defaultIncluded: true },
        { datatypeId: 'percentage', isRequired: false, defaultIncluded: true },
      ],
      functionalities: [
        { id: 'func_premium_calc', name: 'premium_calculation', displayName: 'Premium Calculation', description: 'Core premium calculation functionality', isRequired: true, defaultIncluded: true, requiredComponents: ['customer', 'vehicle', 'pricing'], requiredAbstractAttributes: ['customer_age', 'vehicle_value', 'final_premium'] },
        { id: 'func_discount', name: 'discount_rules', displayName: 'Discount Rules', description: 'Discount calculation based on cover type and loyalty', isRequired: false, defaultIncluded: true, requiredComponents: ['cover', 'pricing'], requiredAbstractAttributes: ['cover_type', 'discount_rate'] },
        { id: 'func_eligibility', name: 'eligibility_check', displayName: 'Eligibility Check', description: 'Customer eligibility validation', isRequired: false, defaultIncluded: true, requiredComponents: ['customer', 'eligibility'], requiredAbstractAttributes: ['customer_age', 'is_eligible'] },
        { id: 'func_ncb', name: 'no_claims_bonus', displayName: 'No Claims Bonus', description: 'NCB calculation and validation', isRequired: false, defaultIncluded: false, requiredComponents: ['customer', 'claims', 'pricing'], requiredAbstractAttributes: ['ncb_years', 'ncb_discount'] },
      ],
      abstractAttributes: [
        { name: 'customer_age', componentId: 'customer', datatypeId: 'integer', description: 'Customer age in years', isInput: true, isRequired: true, defaultIncluded: true, immutable: false, tags: ['input', 'customer'] },
        { name: 'customer_name', componentId: 'customer', datatypeId: 'string', description: 'Customer full name', isInput: true, isRequired: false, defaultIncluded: true, immutable: false, tags: ['input', 'customer'] },
        { name: 'customer_postcode', componentId: 'customer', datatypeId: 'string', description: 'Customer postcode', isInput: true, isRequired: false, defaultIncluded: false, immutable: false, tags: ['input', 'customer', 'location'] },
        { name: 'vehicle_value', componentId: 'vehicle', datatypeId: 'decimal', description: 'Vehicle market value', isInput: true, isRequired: true, defaultIncluded: true, immutable: false, tags: ['input', 'vehicle'] },
        { name: 'vehicle_age', componentId: 'vehicle', datatypeId: 'integer', description: 'Vehicle age in years', isInput: true, isRequired: true, defaultIncluded: true, immutable: false, tags: ['input', 'vehicle'] },
        { name: 'vehicle_type', componentId: 'vehicle', datatypeId: 'enum', enumName: 'vehicle_types', description: 'Type of vehicle', isInput: true, isRequired: false, defaultIncluded: true, immutable: false, tags: ['input', 'vehicle'] },
        { name: 'cover_type', componentId: 'cover', datatypeId: 'enum', enumName: 'cover_types', description: 'Selected cover type', isInput: true, isRequired: true, defaultIncluded: true, immutable: false, tags: ['input', 'cover'] },
        { name: 'base_premium', componentId: 'pricing', datatypeId: 'decimal', description: 'Base premium before adjustments', isInput: false, isRequired: true, defaultIncluded: true, immutable: true, tags: ['output', 'premium'] },
        { name: 'age_factor', componentId: 'pricing', datatypeId: 'decimal', description: 'Age-based premium factor', isInput: false, isRequired: false, defaultIncluded: true, immutable: false, tags: ['output', 'factor'] },
        { name: 'vehicle_factor', componentId: 'pricing', datatypeId: 'decimal', description: 'Vehicle age depreciation factor', isInput: false, isRequired: false, defaultIncluded: true, immutable: false, tags: ['output', 'factor'] },
        { name: 'total_premium', componentId: 'pricing', datatypeId: 'decimal', description: 'Total premium before discount', isInput: false, isRequired: true, defaultIncluded: true, immutable: false, tags: ['output', 'premium'] },
        { name: 'discount_rate', componentId: 'pricing', datatypeId: 'percentage', description: 'Applied discount percentage', isInput: false, isRequired: false, defaultIncluded: true, immutable: false, tags: ['output', 'discount'] },
        { name: 'final_premium', componentId: 'pricing', datatypeId: 'decimal', description: 'Final premium after all adjustments', isInput: false, isRequired: true, defaultIncluded: true, immutable: true, tags: ['output', 'premium'] },
        { name: 'is_eligible', componentId: 'eligibility', datatypeId: 'boolean', description: 'Customer eligibility status', isInput: false, isRequired: false, defaultIncluded: true, immutable: true, tags: ['output', 'eligibility'] },
        { name: 'ncb_years', componentId: 'claims', datatypeId: 'integer', description: 'Years of no claims bonus', isInput: true, isRequired: false, defaultIncluded: false, immutable: false, tags: ['input', 'ncb'] },
        { name: 'ncb_discount', componentId: 'claims', datatypeId: 'percentage', description: 'NCB discount percentage', isInput: false, isRequired: false, defaultIncluded: false, immutable: false, tags: ['output', 'discount', 'ncb'] },
      ],
      createdAt: now() - 86400 * 30,
      updatedAt: now() - 86400 * 7,
    },
    {
      id: 'tpl_health_insurance',
      name: 'Health Insurance',
      description: 'Health insurance product template with medical underwriting and policy benefits',
      type: 'insurance',
      version: '1.0.0',
      enumerations: [
        { id: 'plan_types', name: 'Plan Types', templateType: 'insurance', values: ['basic', 'standard', 'premium', 'platinum'], description: 'Health plan types' },
        { id: 'coverage_areas', name: 'Coverage Areas', templateType: 'insurance', values: ['local', 'national', 'international'], description: 'Geographic coverage' },
      ],
      components: [
        { id: 'member', name: 'member', displayName: 'Member', description: 'Member information and health details', isRequired: true, orderIndex: 0 },
        { id: 'plan', name: 'plan', displayName: 'Plan', description: 'Plan selection and benefits', isRequired: true, orderIndex: 1 },
        { id: 'underwriting', name: 'underwriting', displayName: 'Underwriting', description: 'Medical underwriting and risk assessment', isRequired: false, orderIndex: 2 },
        { id: 'premium', name: 'premium', displayName: 'Premium', description: 'Premium calculation', isRequired: true, orderIndex: 3 },
      ],
      datatypes: [
        { datatypeId: 'integer', isRequired: true, defaultIncluded: true },
        { datatypeId: 'decimal', isRequired: true, defaultIncluded: true },
        { datatypeId: 'string', isRequired: true, defaultIncluded: true },
        { datatypeId: 'boolean', isRequired: true, defaultIncluded: true },
        { datatypeId: 'enum', isRequired: true, defaultIncluded: true },
        { datatypeId: 'date', isRequired: true, defaultIncluded: true },
      ],
      functionalities: [
        { id: 'func_health_premium', name: 'premium_calculation', displayName: 'Premium Calculation', description: 'Health premium calculation', isRequired: true, defaultIncluded: true, requiredComponents: ['member', 'plan', 'premium'], requiredAbstractAttributes: ['member_age', 'plan_type', 'monthly_premium'] },
        { id: 'func_underwrite', name: 'medical_underwriting', displayName: 'Medical Underwriting', description: 'Medical risk assessment', isRequired: false, defaultIncluded: false, requiredComponents: ['member', 'underwriting'], requiredAbstractAttributes: ['member_age', 'pre_existing_conditions', 'risk_score'] },
      ],
      abstractAttributes: [
        { name: 'member_age', componentId: 'member', datatypeId: 'integer', description: 'Member age in years', isInput: true, isRequired: true, defaultIncluded: true, immutable: false, tags: ['input', 'member'] },
        { name: 'member_name', componentId: 'member', datatypeId: 'string', description: 'Member full name', isInput: true, isRequired: true, defaultIncluded: true, immutable: false, tags: ['input', 'member'] },
        { name: 'pre_existing_conditions', componentId: 'member', datatypeId: 'boolean', description: 'Has pre-existing conditions', isInput: true, isRequired: false, defaultIncluded: true, immutable: false, tags: ['input', 'member', 'medical'] },
        { name: 'plan_type', componentId: 'plan', datatypeId: 'enum', enumName: 'plan_types', description: 'Selected health plan', isInput: true, isRequired: true, defaultIncluded: true, immutable: false, tags: ['input', 'plan'] },
        { name: 'coverage_area', componentId: 'plan', datatypeId: 'enum', enumName: 'coverage_areas', description: 'Geographic coverage area', isInput: true, isRequired: false, defaultIncluded: true, immutable: false, tags: ['input', 'plan'] },
        { name: 'risk_score', componentId: 'underwriting', datatypeId: 'integer', description: 'Calculated risk score', isInput: false, isRequired: false, defaultIncluded: false, immutable: false, tags: ['output', 'underwriting'] },
        { name: 'base_premium', componentId: 'premium', datatypeId: 'decimal', description: 'Base premium amount', isInput: false, isRequired: true, defaultIncluded: true, immutable: true, tags: ['output', 'premium'] },
        { name: 'monthly_premium', componentId: 'premium', datatypeId: 'decimal', description: 'Final monthly premium', isInput: false, isRequired: true, defaultIncluded: true, immutable: true, tags: ['output', 'premium'] },
      ],
      createdAt: now() - 86400 * 60,
      updatedAt: now() - 86400 * 14,
    },
    {
      id: 'tpl_loan_product',
      name: 'Loan Product',
      description: 'Consumer loan product template with interest calculation and eligibility rules',
      type: 'lending',
      version: '1.0.0',
      enumerations: [
        { id: 'loan_purposes', name: 'Loan Purposes', templateType: 'lending', values: ['personal', 'auto', 'home-improvement', 'debt-consolidation'], description: 'Loan purpose categories' },
        { id: 'repayment_terms', name: 'Repayment Terms', templateType: 'lending', values: ['12-months', '24-months', '36-months', '48-months', '60-months'], description: 'Loan repayment terms' },
      ],
      components: [
        { id: 'applicant', name: 'applicant', displayName: 'Applicant', description: 'Loan applicant information', isRequired: true, orderIndex: 0 },
        { id: 'loan', name: 'loan', displayName: 'Loan', description: 'Loan details and terms', isRequired: true, orderIndex: 1 },
        { id: 'credit', name: 'credit', displayName: 'Credit', description: 'Credit assessment', isRequired: true, orderIndex: 2 },
        { id: 'repayment', name: 'repayment', displayName: 'Repayment', description: 'Repayment schedule', isRequired: true, orderIndex: 3 },
      ],
      datatypes: [
        { datatypeId: 'integer', isRequired: true, defaultIncluded: true },
        { datatypeId: 'decimal', isRequired: true, defaultIncluded: true },
        { datatypeId: 'string', isRequired: true, defaultIncluded: true },
        { datatypeId: 'boolean', isRequired: true, defaultIncluded: true },
        { datatypeId: 'enum', isRequired: true, defaultIncluded: true },
        { datatypeId: 'percentage', isRequired: true, defaultIncluded: true },
      ],
      functionalities: [
        { id: 'func_loan_calc', name: 'loan_calculation', displayName: 'Loan Calculation', description: 'Calculate loan amounts and interest', isRequired: true, defaultIncluded: true, requiredComponents: ['loan', 'repayment'], requiredAbstractAttributes: ['loan_amount', 'interest_rate', 'monthly_payment'] },
        { id: 'func_credit_check', name: 'credit_assessment', displayName: 'Credit Assessment', description: 'Assess creditworthiness', isRequired: true, defaultIncluded: true, requiredComponents: ['applicant', 'credit'], requiredAbstractAttributes: ['credit_score', 'is_approved'] },
      ],
      abstractAttributes: [
        { name: 'applicant_income', componentId: 'applicant', datatypeId: 'decimal', description: 'Annual income', isInput: true, isRequired: true, defaultIncluded: true, immutable: false, tags: ['input', 'applicant'] },
        { name: 'credit_score', componentId: 'applicant', datatypeId: 'integer', description: 'Credit score', isInput: true, isRequired: true, defaultIncluded: true, immutable: false, tags: ['input', 'credit'] },
        { name: 'loan_amount', componentId: 'loan', datatypeId: 'decimal', description: 'Requested loan amount', isInput: true, isRequired: true, defaultIncluded: true, immutable: false, tags: ['input', 'loan'] },
        { name: 'loan_purpose', componentId: 'loan', datatypeId: 'enum', enumName: 'loan_purposes', description: 'Purpose of the loan', isInput: true, isRequired: true, defaultIncluded: true, immutable: false, tags: ['input', 'loan'] },
        { name: 'loan_term', componentId: 'loan', datatypeId: 'enum', enumName: 'repayment_terms', description: 'Loan term length', isInput: true, isRequired: true, defaultIncluded: true, immutable: false, tags: ['input', 'loan'] },
        { name: 'is_approved', componentId: 'credit', datatypeId: 'boolean', description: 'Loan approval status', isInput: false, isRequired: true, defaultIncluded: true, immutable: true, tags: ['output', 'credit'] },
        { name: 'interest_rate', componentId: 'repayment', datatypeId: 'percentage', description: 'Annual interest rate', isInput: false, isRequired: true, defaultIncluded: true, immutable: false, tags: ['output', 'repayment'] },
        { name: 'monthly_payment', componentId: 'repayment', datatypeId: 'decimal', description: 'Monthly payment amount', isInput: false, isRequired: true, defaultIncluded: true, immutable: true, tags: ['output', 'repayment'] },
      ],
      createdAt: now() - 86400 * 90,
      updatedAt: now() - 86400 * 30,
    },
  ];
}

initMockData();

// =============================================================================
// PRODUCT API
// =============================================================================

export const productApi = {
  async list(pageSize = 20, pageToken = ''): Promise<PaginatedResponse<Product>> {
    if (USE_MOCK) {
      return { items: mockProducts, nextPageToken: '', totalCount: mockProducts.length };
    }
    return fetchJson(`/api/products?pageSize=${pageSize}&pageToken=${pageToken}`);
  },

  async get(id: string): Promise<Product> {
    if (USE_MOCK) {
      const product = mockProducts.find(p => p.id === id);
      if (!product) throw new Error(`Product ${id} not found`);
      return product;
    }
    return fetchJson(`/api/products/${id}`);
  },

  async create(data: Partial<Product>): Promise<Product> {
    if (USE_MOCK) {
      const product: Product = {
        id: data.id || generateId(),
        name: data.name || 'New Product',
        description: data.description || '',
        templateType: data.templateType || 'insurance',
        status: 'DRAFT',
        effectiveFrom: data.effectiveFrom || now(),
        createdAt: now(),
        updatedAt: now(),
        version: 1,
      };
      mockProducts.push(product);
      return product;
    }
    return fetchJson('/api/products', { method: 'POST', body: JSON.stringify(data) });
  },

  async update(id: string, data: Partial<Product>): Promise<Product> {
    if (USE_MOCK) {
      const idx = mockProducts.findIndex(p => p.id === id);
      if (idx < 0) throw new Error(`Product ${id} not found`);
      mockProducts[idx] = { ...mockProducts[idx], ...data, updatedAt: now() };
      return mockProducts[idx];
    }
    return fetchJson(`/api/products/${id}`, { method: 'PUT', body: JSON.stringify(data) });
  },

  async delete(id: string): Promise<void> {
    if (USE_MOCK) {
      mockProducts = mockProducts.filter(p => p.id !== id);
      mockAbstractAttributes = mockAbstractAttributes.filter(a => a.productId !== id);
      mockAttributes = mockAttributes.filter(a => a.productId !== id);
      mockRules = mockRules.filter(r => r.productId !== id);
      mockFunctionalities = mockFunctionalities.filter(f => f.productId !== id);
      return;
    }
    await fetchJson(`/api/products/${id}`, { method: 'DELETE' });
  },

  async clone(request: CloneProductRequest): Promise<CloneProductResponse> {
    if (USE_MOCK) {
      const source = mockProducts.find(p => p.id === request.sourceProductId);
      if (!source) throw new Error(`Product ${request.sourceProductId} not found`);

      const newProductId = generateId();
      const pathMapping: Record<string, string> = {};

      // Clone product
      const clonedProduct: Product = {
        ...source,
        id: newProductId,
        name: request.newProductName,
        description: request.newProductDescription || source.description,
        parentProductId: request.sourceProductId,
        status: 'DRAFT',
        effectiveFrom: request.newEffectiveFrom || now(),
        createdAt: now(),
        updatedAt: now(),
        version: 1,
      };
      mockProducts.push(clonedProduct);

      // Clone abstract attributes
      const sourceAbstractAttrs = mockAbstractAttributes.filter(a => a.productId === request.sourceProductId);
      sourceAbstractAttrs.forEach(attr => {
        const newPath = attr.abstractPath.replace(request.sourceProductId, newProductId);
        pathMapping[attr.abstractPath] = newPath;
        mockAbstractAttributes.push({
          ...attr,
          abstractPath: newPath,
          productId: newProductId,
          immutable: false, // Cloned attributes start as mutable
        });
      });

      // Clone attributes
      const sourceAttrs = mockAttributes.filter(a => a.productId === request.sourceProductId);
      sourceAttrs.forEach(attr => {
        const newPath = attr.path.replace(request.sourceProductId, newProductId);
        const newAbstractPath = attr.abstractPath.replace(request.sourceProductId, newProductId);
        pathMapping[attr.path] = newPath;
        mockAttributes.push({
          ...attr,
          path: newPath,
          abstractPath: newAbstractPath,
          productId: newProductId,
          createdAt: now(),
          updatedAt: now(),
        });
      });

      // Clone rules
      const sourceRules = mockRules.filter(r => r.productId === request.sourceProductId);
      sourceRules.forEach(rule => {
        const newRuleId = generateId();
        pathMapping[rule.id] = newRuleId;
        mockRules.push({
          ...rule,
          id: newRuleId,
          productId: newProductId,
          inputAttributes: rule.inputAttributes.map(ia => ({
            ...ia,
            ruleId: newRuleId,
            attributePath: ia.attributePath.replace(request.sourceProductId, newProductId),
          })),
          outputAttributes: rule.outputAttributes.map(oa => ({
            ...oa,
            ruleId: newRuleId,
            attributePath: oa.attributePath.replace(request.sourceProductId, newProductId),
          })),
          createdAt: now(),
          updatedAt: now(),
        });
      });

      // Clone functionalities
      const sourceFuncs = mockFunctionalities.filter(f => f.productId === request.sourceProductId);
      sourceFuncs.forEach(func => {
        const newFuncId = generateId();
        pathMapping[func.id] = newFuncId;
        mockFunctionalities.push({
          ...func,
          id: newFuncId,
          productId: newProductId,
          status: 'DRAFT',
          immutable: false,
          requiredAttributes: func.requiredAttributes.map(ra => ({
            ...ra,
            functionalityId: newFuncId,
            abstractPath: ra.abstractPath.replace(request.sourceProductId, newProductId),
          })),
          createdAt: now(),
          updatedAt: now(),
        });
      });

      return {
        newProductId,
        abstractAttributesCloned: sourceAbstractAttrs.length,
        attributesCloned: sourceAttrs.length,
        rulesCloned: sourceRules.length,
        functionalitiesCloned: sourceFuncs.length,
        pathMapping,
      };
    }
    return fetchJson(`/api/products/${request.sourceProductId}/clone`, {
      method: 'POST',
      body: JSON.stringify(request),
    });
  },

  async submit(id: string): Promise<Product> {
    if (USE_MOCK) {
      return this.update(id, { status: 'PENDING_APPROVAL' });
    }
    return fetchJson(`/api/products/${id}/submit`, { method: 'POST' });
  },

  async approve(id: string, comments?: string): Promise<Product> {
    if (USE_MOCK) {
      return this.update(id, { status: 'ACTIVE' });
    }
    return fetchJson(`/api/products/${id}/approve`, {
      method: 'POST',
      body: JSON.stringify({ approved: true, comments }),
    });
  },

  async reject(id: string, comments?: string): Promise<Product> {
    if (USE_MOCK) {
      return this.update(id, { status: 'DRAFT' });
    }
    return fetchJson(`/api/products/${id}/approve`, {
      method: 'POST',
      body: JSON.stringify({ approved: false, comments }),
    });
  },

  async discontinue(id: string, reason?: string): Promise<Product> {
    if (USE_MOCK) {
      return this.update(id, { status: 'DISCONTINUED' });
    }
    return fetchJson(`/api/products/${id}/discontinue`, {
      method: 'POST',
      body: JSON.stringify({ reason }),
    });
  },
};

// =============================================================================
// ABSTRACT ATTRIBUTE API
// =============================================================================

export const abstractAttributeApi = {
  async list(productId: string): Promise<AbstractAttribute[]> {
    if (USE_MOCK) {
      return mockAbstractAttributes.filter(a => a.productId === productId);
    }
    const res = await fetchJson<PaginatedResponse<AbstractAttribute>>(
      `/api/products/${productId}/abstract-attributes`
    );
    return res.items;
  },

  async get(path: string): Promise<AbstractAttribute> {
    if (USE_MOCK) {
      const attr = mockAbstractAttributes.find(a => a.abstractPath === path);
      if (!attr) throw new Error(`Attribute ${path} not found`);
      return attr;
    }
    return fetchJson(`/api/abstract-attributes/${encodeURIComponent(path)}`);
  },

  async create(data: Partial<AbstractAttribute>): Promise<AbstractAttribute> {
    if (USE_MOCK) {
      const humanReadableName = (data.attributeName || '').replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
      const attr: AbstractAttribute = {
        abstractPath: data.abstractPath || '',
        productId: data.productId || '',
        componentType: data.componentType || 'default',
        attributeName: data.attributeName || '',
        datatypeId: data.datatypeId || 'string',
        enumName: data.enumName,
        description: data.description || '',
        constraintExpression: data.constraintExpression,
        displayExpression: data.displayExpression || '',
        displayNames: data.displayNames || [
          { name: data.attributeName || '', format: 'SYSTEM', orderIndex: 0 },
          { name: humanReadableName, format: 'HUMAN', orderIndex: 1 },
        ],
        tags: data.tags || [],
        relatedAttributes: data.relatedAttributes || [],
        immutable: data.immutable ?? false,
        createdAt: now(),
        updatedAt: now(),
      };
      mockAbstractAttributes.push(attr);
      return attr;
    }
    return fetchJson('/api/abstract-attributes', { method: 'POST', body: JSON.stringify(data) });
  },

  async update(path: string, data: Partial<AbstractAttribute>): Promise<AbstractAttribute> {
    if (USE_MOCK) {
      const idx = mockAbstractAttributes.findIndex(a => a.abstractPath === path);
      if (idx < 0) throw new Error(`Attribute ${path} not found`);

      // Check immutability
      if (mockAbstractAttributes[idx].immutable) {
        throw new Error(`Cannot modify immutable attribute: ${path}`);
      }

      mockAbstractAttributes[idx] = { ...mockAbstractAttributes[idx], ...data };
      return mockAbstractAttributes[idx];
    }
    return fetchJson(`/api/abstract-attributes/${encodeURIComponent(path)}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  },

  async delete(path: string): Promise<void> {
    if (USE_MOCK) {
      const attr = mockAbstractAttributes.find(a => a.abstractPath === path);
      if (attr?.immutable) {
        throw new Error(`Cannot delete immutable attribute: ${path}`);
      }
      mockAbstractAttributes = mockAbstractAttributes.filter(a => a.abstractPath !== path);
      return;
    }
    await fetchJson(`/api/abstract-attributes/${encodeURIComponent(path)}`, { method: 'DELETE' });
  },

  async getByComponent(productId: string, componentType: string): Promise<AbstractAttribute[]> {
    if (USE_MOCK) {
      return mockAbstractAttributes.filter(
        a => a.productId === productId && a.componentType === componentType
      );
    }
    const res = await fetchJson<PaginatedResponse<AbstractAttribute>>(
      `/api/products/${productId}/abstract-attributes/by-component/${componentType}`
    );
    return res.items;
  },

  async getByTag(productId: string, tag: string): Promise<AbstractAttribute[]> {
    if (USE_MOCK) {
      return mockAbstractAttributes.filter(
        a => a.productId === productId && a.tags.some(t => t.name === tag)
      );
    }
    const res = await fetchJson<PaginatedResponse<AbstractAttribute>>(
      `/api/products/${productId}/abstract-attributes/by-tag/${tag}`
    );
    return res.items;
  },

  async getByFunctionality(productId: string, functionalityId: string): Promise<AbstractAttribute[]> {
    if (USE_MOCK) {
      const func = mockFunctionalities.find(f => f.id === functionalityId && f.productId === productId);
      if (!func) return [];
      const paths = new Set(func.requiredAttributes.map(ra => ra.abstractPath));
      return mockAbstractAttributes.filter(a => paths.has(a.abstractPath));
    }
    const res = await fetchJson<PaginatedResponse<AbstractAttribute>>(
      `/api/products/${productId}/functionalities/${functionalityId}/abstract-attributes`
    );
    return res.items;
  },
};

// =============================================================================
// ATTRIBUTE API
// =============================================================================

export const attributeApi = {
  async list(productId: string): Promise<Attribute[]> {
    if (USE_MOCK) {
      return mockAttributes.filter(a => a.productId === productId);
    }
    const res = await fetchJson<PaginatedResponse<Attribute>>(
      `/api/products/${productId}/attributes`
    );
    return res.items;
  },

  async get(path: string): Promise<Attribute> {
    if (USE_MOCK) {
      const attr = mockAttributes.find(a => a.path === path);
      if (!attr) throw new Error(`Attribute ${path} not found`);
      return attr;
    }
    return fetchJson(`/api/attributes/${encodeURIComponent(path)}`);
  },

  async create(data: Partial<Attribute>): Promise<Attribute> {
    if (USE_MOCK) {
      const attr: Attribute = {
        path: data.path || '',
        abstractPath: data.abstractPath || '',
        productId: data.productId || '',
        componentType: data.componentType || 'default',
        componentId: data.componentId || 'default',
        attributeName: data.attributeName || '',
        valueType: data.valueType || 'FIXED_VALUE',
        value: data.value,
        ruleId: data.ruleId,
        createdAt: now(),
        updatedAt: now(),
      };
      mockAttributes.push(attr);
      return attr;
    }
    return fetchJson('/api/attributes', { method: 'POST', body: JSON.stringify(data) });
  },

  async update(path: string, data: Partial<Attribute>): Promise<Attribute> {
    if (USE_MOCK) {
      const idx = mockAttributes.findIndex(a => a.path === path);
      if (idx < 0) throw new Error(`Attribute ${path} not found`);

      // Check if abstract attribute is immutable
      const abstractAttr = mockAbstractAttributes.find(a => a.abstractPath === mockAttributes[idx].abstractPath);
      if (abstractAttr?.immutable) {
        throw new Error(`Cannot modify value of immutable attribute: ${path}`);
      }

      mockAttributes[idx] = { ...mockAttributes[idx], ...data, updatedAt: now() };
      return mockAttributes[idx];
    }
    return fetchJson(`/api/attributes/${encodeURIComponent(path)}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  },

  async delete(path: string): Promise<void> {
    if (USE_MOCK) {
      mockAttributes = mockAttributes.filter(a => a.path !== path);
      return;
    }
    await fetchJson(`/api/attributes/${encodeURIComponent(path)}`, { method: 'DELETE' });
  },

  async getByTag(productId: string, tag: string): Promise<Attribute[]> {
    if (USE_MOCK) {
      const abstractPaths = new Set(
        mockAbstractAttributes
          .filter(a => a.productId === productId && a.tags.some(t => t.name === tag))
          .map(a => a.abstractPath)
      );
      return mockAttributes.filter(a => abstractPaths.has(a.abstractPath));
    }
    const res = await fetchJson<PaginatedResponse<Attribute>>(
      `/api/products/${productId}/attributes/by-tag/${tag}`
    );
    return res.items;
  },

  async getByFunctionality(productId: string, functionalityId: string): Promise<Attribute[]> {
    if (USE_MOCK) {
      const func = mockFunctionalities.find(f => f.id === functionalityId && f.productId === productId);
      if (!func) return [];
      const abstractPaths = new Set(func.requiredAttributes.map(ra => ra.abstractPath));
      return mockAttributes.filter(a => abstractPaths.has(a.abstractPath));
    }
    const res = await fetchJson<PaginatedResponse<Attribute>>(
      `/api/products/${productId}/functionalities/${functionalityId}/attributes`
    );
    return res.items;
  },
};

// =============================================================================
// RULE API
// =============================================================================

export const ruleApi = {
  async list(productId: string): Promise<Rule[]> {
    if (USE_MOCK) {
      return mockRules.filter(r => r.productId === productId);
    }
    const res = await fetchJson<PaginatedResponse<Rule>>(
      `/api/products/${productId}/rules`
    );
    return res.items;
  },

  async get(id: string): Promise<Rule> {
    if (USE_MOCK) {
      const rule = mockRules.find(r => r.id === id);
      if (!rule) throw new Error(`Rule ${id} not found`);
      return rule;
    }
    return fetchJson(`/api/rules/${id}`);
  },

  async create(data: Partial<Rule>): Promise<Rule> {
    if (USE_MOCK) {
      // Check if any output attributes are immutable
      const outputPaths = data.outputAttributes?.map(o => o.attributePath) || [];
      for (const path of outputPaths) {
        const attr = mockAbstractAttributes.find(a => a.abstractPath === path);
        if (attr?.immutable) {
          throw new Error(`Cannot create rule targeting immutable attribute: ${path}`);
        }
      }

      const id = generateId();
      const rule: Rule = {
        id,
        productId: data.productId || '',
        ruleType: data.ruleType || 'calculation',
        displayExpression: data.displayExpression || '',
        compiledExpression: data.compiledExpression || '{}',
        description: data.description,
        enabled: data.enabled ?? true,
        orderIndex: data.orderIndex ?? mockRules.length,
        inputAttributes: data.inputAttributes || [],
        outputAttributes: data.outputAttributes || [],
        createdAt: now(),
        updatedAt: now(),
      };
      mockRules.push(rule);
      return rule;
    }
    return fetchJson('/api/rules', { method: 'POST', body: JSON.stringify(data) });
  },

  async update(id: string, data: Partial<Rule>): Promise<Rule> {
    if (USE_MOCK) {
      const idx = mockRules.findIndex(r => r.id === id);
      if (idx < 0) throw new Error(`Rule ${id} not found`);

      // Check if any output attributes are immutable
      const outputPaths = (data.outputAttributes || mockRules[idx].outputAttributes).map(o => o.attributePath);
      for (const path of outputPaths) {
        const attr = mockAbstractAttributes.find(a => a.abstractPath === path);
        if (attr?.immutable) {
          throw new Error(`Cannot modify rule targeting immutable attribute: ${path}. Clone the product first.`);
        }
      }

      mockRules[idx] = { ...mockRules[idx], ...data, updatedAt: now() };
      return mockRules[idx];
    }
    return fetchJson(`/api/rules/${id}`, { method: 'PUT', body: JSON.stringify(data) });
  },

  async delete(id: string): Promise<void> {
    if (USE_MOCK) {
      const rule = mockRules.find(r => r.id === id);
      if (rule) {
        // Check if any output attributes are immutable
        for (const oa of rule.outputAttributes) {
          const attr = mockAbstractAttributes.find(a => a.abstractPath === oa.attributePath);
          if (attr?.immutable) {
            throw new Error(`Cannot delete rule targeting immutable attribute: ${oa.attributePath}`);
          }
        }
      }
      mockRules = mockRules.filter(r => r.id !== id);
      return;
    }
    await fetchJson(`/api/rules/${id}`, { method: 'DELETE' });
  },

  async getByFunctionality(productId: string, functionalityId: string): Promise<Rule[]> {
    if (USE_MOCK) {
      const func = mockFunctionalities.find(f => f.id === functionalityId && f.productId === productId);
      if (!func) return [];
      const funcPaths = new Set(func.requiredAttributes.map(ra => ra.abstractPath));

      // Find rules that have outputs in functionality
      return mockRules.filter(r =>
        r.productId === productId &&
        r.outputAttributes.some(oa => funcPaths.has(oa.attributePath))
      );
    }
    const res = await fetchJson<PaginatedResponse<Rule>>(
      `/api/products/${productId}/functionalities/${functionalityId}/rules`
    );
    return res.items;
  },
};

// =============================================================================
// FUNCTIONALITY API
// =============================================================================

export const functionalityApi = {
  async list(productId: string): Promise<ProductFunctionality[]> {
    if (USE_MOCK) {
      return mockFunctionalities.filter(f => f.productId === productId);
    }
    const res = await fetchJson<PaginatedResponse<ProductFunctionality>>(
      `/api/products/${productId}/functionalities`
    );
    return res.items;
  },

  async get(productId: string, id: string): Promise<ProductFunctionality> {
    if (USE_MOCK) {
      const func = mockFunctionalities.find(f => f.id === id && f.productId === productId);
      if (!func) throw new Error(`Functionality ${id} not found`);
      return func;
    }
    return fetchJson(`/api/products/${productId}/functionalities/${id}`);
  },

  async create(data: Partial<ProductFunctionality>): Promise<ProductFunctionality> {
    if (USE_MOCK) {
      const func: ProductFunctionality = {
        id: data.id || generateId(),
        productId: data.productId || '',
        name: data.name || '',
        displayName: data.displayName || '',
        description: data.description || '',
        status: 'DRAFT',
        immutable: false,
        requiredAttributes: data.requiredAttributes || [],
        createdAt: now(),
        updatedAt: now(),
      };
      mockFunctionalities.push(func);
      return func;
    }
    return fetchJson('/api/functionalities', { method: 'POST', body: JSON.stringify(data) });
  },

  async update(productId: string, id: string, data: Partial<ProductFunctionality>): Promise<ProductFunctionality> {
    if (USE_MOCK) {
      const idx = mockFunctionalities.findIndex(f => f.id === id && f.productId === productId);
      if (idx < 0) throw new Error(`Functionality ${id} not found`);

      if (mockFunctionalities[idx].immutable) {
        throw new Error(`Cannot modify immutable functionality: ${id}`);
      }

      mockFunctionalities[idx] = { ...mockFunctionalities[idx], ...data, updatedAt: now() };
      return mockFunctionalities[idx];
    }
    return fetchJson(`/api/products/${productId}/functionalities/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  },

  async delete(productId: string, id: string): Promise<void> {
    if (USE_MOCK) {
      const func = mockFunctionalities.find(f => f.id === id && f.productId === productId);
      if (func?.immutable) {
        throw new Error(`Cannot delete immutable functionality: ${id}`);
      }
      mockFunctionalities = mockFunctionalities.filter(f => !(f.id === id && f.productId === productId));
      return;
    }
    await fetchJson(`/api/products/${productId}/functionalities/${id}`, { method: 'DELETE' });
  },

  async addRequiredAttribute(
    productId: string,
    functionalityId: string,
    attribute: FunctionalityRequiredAttribute
  ): Promise<ProductFunctionality> {
    if (USE_MOCK) {
      const idx = mockFunctionalities.findIndex(f => f.id === functionalityId && f.productId === productId);
      if (idx < 0) throw new Error(`Functionality ${functionalityId} not found`);

      if (mockFunctionalities[idx].immutable) {
        throw new Error(`Cannot modify immutable functionality: ${functionalityId}`);
      }

      mockFunctionalities[idx].requiredAttributes.push(attribute);
      mockFunctionalities[idx].updatedAt = now();
      return mockFunctionalities[idx];
    }
    return fetchJson(`/api/products/${productId}/functionalities/${functionalityId}/required-attributes`, {
      method: 'POST',
      body: JSON.stringify(attribute),
    });
  },

  async removeRequiredAttribute(
    productId: string,
    functionalityId: string,
    abstractPath: string
  ): Promise<ProductFunctionality> {
    if (USE_MOCK) {
      const idx = mockFunctionalities.findIndex(f => f.id === functionalityId && f.productId === productId);
      if (idx < 0) throw new Error(`Functionality ${functionalityId} not found`);

      if (mockFunctionalities[idx].immutable) {
        throw new Error(`Cannot modify immutable functionality: ${functionalityId}`);
      }

      mockFunctionalities[idx].requiredAttributes = mockFunctionalities[idx].requiredAttributes.filter(
        ra => ra.abstractPath !== abstractPath
      );
      mockFunctionalities[idx].updatedAt = now();
      return mockFunctionalities[idx];
    }
    return fetchJson(
      `/api/products/${productId}/functionalities/${functionalityId}/required-attributes/${encodeURIComponent(abstractPath)}`,
      { method: 'DELETE' }
    );
  },

  async submit(productId: string, id: string): Promise<ProductFunctionality> {
    if (USE_MOCK) {
      return this.update(productId, id, { status: 'PENDING_APPROVAL' });
    }
    return fetchJson(`/api/products/${productId}/functionalities/${id}/submit`, { method: 'POST' });
  },

  async approve(productId: string, id: string, comments?: string): Promise<ProductFunctionality> {
    if (USE_MOCK) {
      return this.update(productId, id, { status: 'ACTIVE' });
    }
    return fetchJson(`/api/products/${productId}/functionalities/${id}/approve`, {
      method: 'POST',
      body: JSON.stringify({ approved: true, comments }),
    });
  },

  async reject(productId: string, id: string, comments?: string): Promise<ProductFunctionality> {
    if (USE_MOCK) {
      return this.update(productId, id, { status: 'DRAFT' });
    }
    return fetchJson(`/api/products/${productId}/functionalities/${id}/approve`, {
      method: 'POST',
      body: JSON.stringify({ approved: false, comments }),
    });
  },

  async evaluate(request: EvaluateFunctionalityRequest): Promise<EvaluateFunctionalityResponse> {
    if (USE_MOCK) {
      const func = mockFunctionalities.find(
        f => f.id === request.functionalityId && f.productId === request.productId
      );
      if (!func) throw new Error(`Functionality ${request.functionalityId} not found`);

      // Get required attribute paths
      const requiredPaths = new Set(func.requiredAttributes.map(ra => ra.abstractPath));

      // Find rules that output to functionality attributes
      const relevantRules = mockRules.filter(
        r => r.productId === request.productId &&
          r.outputAttributes.some(oa => requiredPaths.has(oa.attributePath))
      );

      // Run full evaluation and filter outputs
      const fullResult = await evaluationApi.evaluate({
        productId: request.productId,
        inputData: request.inputData,
        ruleIds: relevantRules.map(r => r.id),
        options: request.options,
      });

      // Filter to only include functionality outputs
      const outputs: Record<string, AttributeValue> = {};
      const missingRequired: string[] = [];

      func.requiredAttributes.forEach(ra => {
        const attrName = ra.abstractPath.split(':').pop() || '';
        if (fullResult.outputs[attrName]) {
          outputs[attrName] = fullResult.outputs[attrName];
        } else {
          missingRequired.push(ra.abstractPath);
        }
      });

      return {
        success: fullResult.success && missingRequired.length === 0,
        functionalityId: func.id,
        functionalityName: func.displayName,
        outputs,
        missingRequiredAttributes: missingRequired,
        metrics: fullResult.metrics,
        errors: fullResult.errors,
      };
    }
    return fetchJson('/api/evaluate-functionality', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  },
};

// =============================================================================
// DATATYPE API
// =============================================================================

export const datatypeApi = {
  async list(): Promise<DataType[]> {
    if (USE_MOCK) {
      return mockDatatypes;
    }
    const res = await fetchJson<PaginatedResponse<DataType>>('/api/datatypes');
    return res.items;
  },

  async get(id: string): Promise<DataType> {
    if (USE_MOCK) {
      const dt = mockDatatypes.find(d => d.id === id);
      if (!dt) throw new Error(`Datatype ${id} not found`);
      return dt;
    }
    return fetchJson(`/api/datatypes/${id}`);
  },

  async create(data: Partial<DataType>): Promise<DataType> {
    if (USE_MOCK) {
      const dt: DataType = {
        id: data.id || generateId(),
        name: data.name || '',
        primitiveType: data.primitiveType || 'STRING',
        constraints: data.constraints || {},
        constraintsJson: data.constraintsJson,
        description: data.description,
        createdAt: now(),
        updatedAt: now(),
      };
      mockDatatypes.push(dt);
      return dt;
    }
    return fetchJson('/api/datatypes', { method: 'POST', body: JSON.stringify(data) });
  },

  async update(id: string, data: Partial<DataType>): Promise<DataType> {
    if (USE_MOCK) {
      const idx = mockDatatypes.findIndex(d => d.id === id);
      if (idx < 0) throw new Error(`Datatype ${id} not found`);
      mockDatatypes[idx] = { ...mockDatatypes[idx], ...data };
      return mockDatatypes[idx];
    }
    return fetchJson(`/api/datatypes/${id}`, { method: 'PUT', body: JSON.stringify(data) });
  },

  async delete(id: string): Promise<void> {
    if (USE_MOCK) {
      // Check if datatype is in use
      const inUse = mockAbstractAttributes.some(a => a.datatypeId === id);
      if (inUse) {
        throw new Error(`Cannot delete datatype ${id}: still in use by attributes`);
      }
      mockDatatypes = mockDatatypes.filter(d => d.id !== id);
      return;
    }
    await fetchJson(`/api/datatypes/${id}`, { method: 'DELETE' });
  },
};

// =============================================================================
// TEMPLATE API
// =============================================================================

export const templateApi = {
  // =============================================
  // PRODUCT TEMPLATES
  // =============================================

  async listProductTemplates(): Promise<ProductTemplate[]> {
    if (USE_MOCK) {
      return mockProductTemplates;
    }
    const res = await fetchJson<PaginatedResponse<ProductTemplate>>('/api/product-templates');
    return res.items;
  },

  async getProductTemplate(id: string): Promise<ProductTemplate> {
    if (USE_MOCK) {
      const template = mockProductTemplates.find(t => t.id === id);
      if (!template) throw new Error(`Template ${id} not found`);
      return template;
    }
    return fetchJson(`/api/product-templates/${id}`);
  },

  async createProductFromTemplate(
    templateId: string,
    productInfo: {
      id: string;
      name: string;
      description: string;
      effectiveFrom: number;
      expiryAt?: number;
    },
    selectedComponents: string[],
    selectedDatatypes: string[],
    selectedEnumerations: string[],
    selectedFunctionalities: string[],
    selectedAbstractAttributes: string[]
  ): Promise<{
    product: Product;
    abstractAttributesCreated: number;
    functionalitiesCreated: number;
    enumerationsCreated: number;
  }> {
    if (USE_MOCK) {
      const template = mockProductTemplates.find(t => t.id === templateId);
      if (!template) throw new Error(`Template ${templateId} not found`);

      // Create the product
      const product: Product = {
        id: productInfo.id,
        name: productInfo.name,
        description: productInfo.description,
        templateType: template.type,
        status: 'DRAFT',
        effectiveFrom: productInfo.effectiveFrom,
        expiryAt: productInfo.expiryAt,
        createdAt: now(),
        updatedAt: now(),
        version: 1,
      };
      mockProducts.push(product);

      // Create selected enumerations
      const selectedEnumSet = new Set(selectedEnumerations);
      const enumerationsToCreate = template.enumerations.filter(e => selectedEnumSet.has(e.id));
      enumerationsToCreate.forEach(e => {
        if (!mockTemplateEnumerations.find(existing => existing.id === e.id)) {
          mockTemplateEnumerations.push({ ...e });
        }
      });

      // Create selected abstract attributes
      const selectedAttrSet = new Set(selectedAbstractAttributes);
      const selectedCompSet = new Set(selectedComponents);
      let abstractAttributesCreated = 0;

      template.abstractAttributes
        .filter(a => selectedAttrSet.has(a.name) && selectedCompSet.has(a.componentId))
        .forEach(attr => {
          const abstractPath = `${product.id}:abstract-path:${attr.componentId}:${attr.name}`;
          const humanReadableName = attr.name.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());

          mockAbstractAttributes.push({
            abstractPath,
            productId: product.id,
            componentType: attr.componentId,
            attributeName: attr.name,
            datatypeId: attr.datatypeId,
            enumName: attr.enumName,
            description: attr.description,
            displayExpression: attr.name.replace(/_/g, ' '),
            displayNames: [
              { name: attr.name, format: 'SYSTEM', orderIndex: 0 },
              { name: humanReadableName, format: 'HUMAN', orderIndex: 1 },
            ],
            tags: attr.tags.map((t, i) => ({ name: t, orderIndex: i })),
            relatedAttributes: [],
            immutable: attr.immutable,
            createdAt: now(),
            updatedAt: now(),
          });

          // Create corresponding attribute instance
          mockAttributes.push({
            path: `${product.id}:${attr.componentId}:default:${attr.name}`,
            abstractPath,
            productId: product.id,
            componentType: attr.componentId,
            componentId: 'default',
            attributeName: attr.name,
            valueType: attr.isInput ? 'JUST_DEFINITION' : 'RULE_DRIVEN',
            value: attr.isInput ? undefined : { type: 'null' },
            createdAt: now(),
            updatedAt: now(),
          });

          abstractAttributesCreated++;
        });

      // Create selected functionalities
      const selectedFuncSet = new Set(selectedFunctionalities);
      let functionalitiesCreated = 0;

      template.functionalities
        .filter(f => selectedFuncSet.has(f.id))
        .forEach(func => {
          const funcId = generateId();
          const requiredAttributes: FunctionalityRequiredAttribute[] = func.requiredAbstractAttributes
            .filter(attrName => selectedAttrSet.has(attrName))
            .map((attrName, idx) => {
              const attr = template.abstractAttributes.find(a => a.name === attrName);
              return {
                functionalityId: funcId,
                abstractPath: `${product.id}:abstract-path:${attr?.componentId || 'default'}:${attrName}`,
                description: attr?.description || '',
                orderIndex: idx,
              };
            });

          mockFunctionalities.push({
            id: funcId,
            productId: product.id,
            name: func.name,
            displayName: func.displayName,
            description: func.description,
            status: 'DRAFT',
            immutable: false,
            requiredAttributes,
            createdAt: now(),
            updatedAt: now(),
          });

          functionalitiesCreated++;
        });

      return {
        product,
        abstractAttributesCreated,
        functionalitiesCreated,
        enumerationsCreated: enumerationsToCreate.length,
      };
    }

    return fetchJson('/api/products/from-template', {
      method: 'POST',
      body: JSON.stringify({
        templateId,
        productInfo,
        selectedComponents,
        selectedDatatypes,
        selectedEnumerations,
        selectedFunctionalities,
        selectedAbstractAttributes,
      }),
    });
  },

  // =============================================
  // TEMPLATE ENUMERATIONS
  // =============================================

  async listEnumerations(templateType?: string): Promise<TemplateEnumeration[]> {
    if (USE_MOCK) {
      if (templateType) {
        return mockTemplateEnumerations.filter(e => e.templateType === templateType);
      }
      return mockTemplateEnumerations;
    }
    const url = templateType
      ? `/api/template-enumerations?templateType=${templateType}`
      : '/api/template-enumerations';
    const res = await fetchJson<PaginatedResponse<TemplateEnumeration>>(url);
    return res.items;
  },

  async getEnumeration(id: string): Promise<TemplateEnumeration> {
    if (USE_MOCK) {
      const en = mockTemplateEnumerations.find(e => e.id === id);
      if (!en) throw new Error(`Template enumeration ${id} not found`);
      return en;
    }
    return fetchJson(`/api/template-enumerations/${id}`);
  },

  async createEnumeration(data: Partial<TemplateEnumeration>): Promise<TemplateEnumeration> {
    if (USE_MOCK) {
      const en: TemplateEnumeration = {
        id: data.id || generateId(),
        name: data.name || '',
        templateType: data.templateType || '',
        values: data.values || [],
        description: data.description,
      };
      mockTemplateEnumerations.push(en);
      return en;
    }
    return fetchJson('/api/template-enumerations', { method: 'POST', body: JSON.stringify(data) });
  },

  async updateEnumeration(id: string, data: Partial<TemplateEnumeration>): Promise<TemplateEnumeration> {
    if (USE_MOCK) {
      const idx = mockTemplateEnumerations.findIndex(e => e.id === id);
      if (idx < 0) throw new Error(`Template enumeration ${id} not found`);
      mockTemplateEnumerations[idx] = { ...mockTemplateEnumerations[idx], ...data };
      return mockTemplateEnumerations[idx];
    }
    return fetchJson(`/api/template-enumerations/${id}`, { method: 'PUT', body: JSON.stringify(data) });
  },

  async deleteEnumeration(id: string): Promise<void> {
    if (USE_MOCK) {
      mockTemplateEnumerations = mockTemplateEnumerations.filter(e => e.id !== id);
      return;
    }
    await fetchJson(`/api/template-enumerations/${id}`, { method: 'DELETE' });
  },

  async addValue(id: string, value: string): Promise<TemplateEnumeration> {
    if (USE_MOCK) {
      const idx = mockTemplateEnumerations.findIndex(e => e.id === id);
      if (idx < 0) throw new Error(`Template enumeration ${id} not found`);

      // Validate value format (lowercase letters and hyphens only)
      const pattern = /^[a-z]([-][a-z]|[a-z]){0,50}$/;
      if (!pattern.test(value)) {
        throw new Error(`Invalid value format: ${value}. Must match pattern: lowercase letters and hyphens.`);
      }

      // Check for duplicate
      if (mockTemplateEnumerations[idx].values.includes(value)) {
        throw new Error(`Value '${value}' already exists in enumeration`);
      }

      mockTemplateEnumerations[idx].values.push(value);
      return mockTemplateEnumerations[idx];
    }
    return fetchJson(`/api/template-enumerations/${id}/values`, {
      method: 'POST',
      body: JSON.stringify({ value }),
    });
  },

  async removeValue(id: string, value: string): Promise<TemplateEnumeration> {
    if (USE_MOCK) {
      const idx = mockTemplateEnumerations.findIndex(e => e.id === id);
      if (idx < 0) throw new Error(`Template enumeration ${id} not found`);

      // Check if value exists
      if (!mockTemplateEnumerations[idx].values.includes(value)) {
        throw new Error(`Value '${value}' not found in enumeration`);
      }

      // Check if value is in use by any abstract attribute
      const inUse = mockAbstractAttributes.some(
        a => a.enumName === mockTemplateEnumerations[idx].name &&
             a.datatypeId === 'enum'
      );
      if (inUse) {
        // Just a warning - still allow removal but could check concrete values
        console.warn(`Warning: Enumeration ${id} is in use by attributes`);
      }

      mockTemplateEnumerations[idx].values = mockTemplateEnumerations[idx].values.filter(v => v !== value);
      return mockTemplateEnumerations[idx];
    }
    return fetchJson(`/api/template-enumerations/${id}/values/${encodeURIComponent(value)}`, {
      method: 'DELETE',
    });
  },

  async reorderValues(id: string, values: string[]): Promise<TemplateEnumeration> {
    if (USE_MOCK) {
      const idx = mockTemplateEnumerations.findIndex(e => e.id === id);
      if (idx < 0) throw new Error(`Template enumeration ${id} not found`);

      // Validate that values array contains same values
      const existingSet = new Set(mockTemplateEnumerations[idx].values);
      const newSet = new Set(values);
      if (existingSet.size !== newSet.size ||
          ![...existingSet].every(v => newSet.has(v))) {
        throw new Error('Reordered values must contain exactly the same values as original');
      }

      mockTemplateEnumerations[idx].values = values;
      return mockTemplateEnumerations[idx];
    }
    return fetchJson(`/api/template-enumerations/${id}/reorder`, {
      method: 'POST',
      body: JSON.stringify({ values }),
    });
  },
};

// =============================================================================
// EVALUATION API
// =============================================================================

export const evaluationApi = {
  async evaluate(request: EvaluateRequest): Promise<EvaluateResponse> {
    if (USE_MOCK) {
      // Simple mock evaluation
      const outputs: Record<string, AttributeValue> = {};
      const inputs = request.inputData;

      // Execute rules in order (simplified)
      const rules = mockRules.filter(r => r.productId === request.productId && r.enabled);

      // For demo, just compute based on inputs
      const vehicleValue = (inputs['vehicle_value'] as { value: number })?.value || 50000;
      const customerAge = (inputs['customer_age'] as { value: number })?.value || 35;
      const vehicleAge = (inputs['vehicle_age'] as { value: number })?.value || 3;
      const coverType = (inputs['cover_type'] as { value: string })?.value || 'comprehensive';

      const basePremium = vehicleValue * 0.05;
      const ageFactor = customerAge < 25 ? 1.5 : customerAge > 65 ? 1.3 : 1.0;
      const vehicleFactor = vehicleAge > 10 ? 0.7 : vehicleAge > 5 ? 0.85 : 1.0;
      const totalPremium = basePremium * ageFactor * vehicleFactor;
      const discountRate = coverType === 'comprehensive' ? 0.1 : 0.05;
      const finalPremium = totalPremium * (1 - discountRate);
      const isEligible = customerAge >= 18 && customerAge <= 85;

      outputs['base_premium'] = { type: 'float', value: basePremium };
      outputs['age_factor'] = { type: 'float', value: ageFactor };
      outputs['vehicle_factor'] = { type: 'float', value: vehicleFactor };
      outputs['total_premium'] = { type: 'float', value: totalPremium };
      outputs['discount_rate'] = { type: 'float', value: discountRate };
      outputs['final_premium'] = { type: 'float', value: finalPremium };
      outputs['is_eligible'] = { type: 'bool', value: isEligible };

      return {
        success: true,
        outputs,
        ruleResults: rules.map(r => ({
          ruleId: r.id,
          outputs: [],
          executionTimeNs: Math.floor(Math.random() * 100000),
          skipped: false,
        })),
        metrics: {
          totalTimeNs: Math.floor(Math.random() * 1000000),
          rulesExecuted: rules.length,
          rulesSkipped: 0,
          cacheHits: 0,
          levels: [],
        },
        errors: [],
      };
    }
    return fetchJson('/api/evaluate', { method: 'POST', body: JSON.stringify(request) });
  },

  async batchEvaluate(request: BatchEvaluateRequest): Promise<BatchEvaluateResponse> {
    if (USE_MOCK) {
      const startTime = Date.now();
      const results = await Promise.all(request.requests.map(r => this.evaluate(r)));
      const totalTimeMs = Date.now() - startTime;
      const successCount = results.filter(r => r.success).length;
      return {
        results,
        metrics: {
          totalRequests: results.length,
          successCount,
          failureCount: results.length - successCount,
          totalTimeMs,
          avgTimePerRequest: totalTimeMs / results.length,
        },
      };
    }
    return fetchJson('/api/batch-evaluate', { method: 'POST', body: JSON.stringify(request) });
  },

  async getExecutionPlan(productId: string, ruleIds?: string[]): Promise<ExecutionPlan> {
    if (USE_MOCK) {
      const rules = mockRules.filter(r => r.productId === productId);

      // Build dependency graph
      const outputToRule = new Map<string, string>();
      rules.forEach(r => {
        r.outputAttributes.forEach(o => outputToRule.set(o.attributePath, r.id));
      });

      const dependencies: { ruleId: string; dependsOn: string[] }[] = [];
      rules.forEach(r => {
        const deps: string[] = [];
        r.inputAttributes.forEach(i => {
          const depRule = outputToRule.get(i.attributePath);
          if (depRule && depRule !== r.id) deps.push(depRule);
        });
        dependencies.push({ ruleId: r.id, dependsOn: deps });
      });

      // Simple topological levels
      const levels: { level: number; ruleIds: string[] }[] = [];
      const assigned = new Set<string>();
      let level = 0;

      while (assigned.size < rules.length && level < 10) {
        const levelRules: string[] = [];
        dependencies.forEach(d => {
          if (!assigned.has(d.ruleId) && d.dependsOn.every(dep => assigned.has(dep))) {
            levelRules.push(d.ruleId);
          }
        });
        if (levelRules.length === 0) break;
        levelRules.forEach(r => assigned.add(r));
        levels.push({ level, ruleIds: levelRules });
        level++;
      }

      return {
        levels,
        dependencies,
        missingInputs: [],
        hasCycles: assigned.size < rules.length,
      };
    }
    return fetchJson(`/api/products/${productId}/execution-plan`, {
      method: 'POST',
      body: JSON.stringify({ ruleIds }),
    });
  },

  async validate(productId: string): Promise<ValidationResult> {
    if (USE_MOCK) {
      // Check for basic validation issues
      const errors: { code: string; message: string; path?: string; ruleId?: string }[] = [];
      const warnings: { code: string; message: string; path?: string; ruleId?: string }[] = [];

      const rules = mockRules.filter(r => r.productId === productId);
      const attrs = mockAbstractAttributes.filter(a => a.productId === productId);

      // Check for orphan rules (no outputs)
      rules.forEach(r => {
        if (r.outputAttributes.length === 0) {
          warnings.push({ code: 'ORPHAN_RULE', message: 'Rule has no output attributes', ruleId: r.id });
        }
      });

      // Check for missing input attributes
      const definedPaths = new Set(attrs.map(a => a.abstractPath));
      rules.forEach(r => {
        r.inputAttributes.forEach(ia => {
          if (!definedPaths.has(ia.attributePath)) {
            errors.push({
              code: 'MISSING_INPUT',
              message: `Input attribute not defined: ${ia.attributePath}`,
              ruleId: r.id,
            });
          }
        });
      });

      return { isValid: errors.length === 0, errors, warnings };
    }
    return fetchJson(`/api/products/${productId}/validate`);
  },
};

// =============================================================================
// IMPACT ANALYSIS API
// =============================================================================

export const impactApi = {
  async analyze(productId: string, targetPath: string): Promise<ImpactAnalysis> {
    if (USE_MOCK) {
      const rules = mockRules.filter(r => r.productId === productId);
      const attrs = mockAbstractAttributes.filter(a => a.productId === productId);
      const funcs = mockFunctionalities.filter(f => f.productId === productId);

      // Build dependency graph
      const outputToRule = new Map<string, string>();
      const inputToRules = new Map<string, string[]>();
      const ruleToOutputs = new Map<string, string[]>();
      const ruleToInputs = new Map<string, string[]>();

      rules.forEach(r => {
        ruleToOutputs.set(r.id, r.outputAttributes.map(o => o.attributePath));
        ruleToInputs.set(r.id, r.inputAttributes.map(i => i.attributePath));
        r.outputAttributes.forEach(o => outputToRule.set(o.attributePath, r.id));
        r.inputAttributes.forEach(i => {
          const existing = inputToRules.get(i.attributePath) || [];
          existing.push(r.id);
          inputToRules.set(i.attributePath, existing);
        });
      });

      // Find direct upstream (attributes this one depends on)
      const directUpstream: DependencyInfo[] = [];
      const rule = outputToRule.get(targetPath);
      if (rule) {
        const inputs = ruleToInputs.get(rule) || [];
        inputs.forEach(path => {
          const attr = attrs.find(a => a.abstractPath === path);
          if (attr) {
            directUpstream.push({
              path,
              attributeName: attr.attributeName,
              direction: 'upstream',
              distance: 1,
              isImmutable: attr.immutable,
            });
          }
        });
      }

      // Find direct downstream (attributes that depend on this one)
      const directDownstream: DependencyInfo[] = [];
      const dependentRules = inputToRules.get(targetPath) || [];
      dependentRules.forEach(ruleId => {
        const outputs = ruleToOutputs.get(ruleId) || [];
        outputs.forEach(path => {
          const attr = attrs.find(a => a.abstractPath === path);
          if (attr) {
            directDownstream.push({
              path,
              attributeName: attr.attributeName,
              direction: 'downstream',
              distance: 1,
              isImmutable: attr.immutable,
            });
          }
        });
      });

      // Find transitive dependencies (simplified - just one more level)
      const transitiveDownstream: DependencyInfo[] = [];
      directDownstream.forEach(dep => {
        const nextRules = inputToRules.get(dep.path) || [];
        nextRules.forEach(ruleId => {
          const outputs = ruleToOutputs.get(ruleId) || [];
          outputs.forEach(path => {
            if (path !== targetPath && !directDownstream.some(d => d.path === path)) {
              const attr = attrs.find(a => a.abstractPath === path);
              if (attr) {
                transitiveDownstream.push({
                  path,
                  attributeName: attr.attributeName,
                  direction: 'downstream',
                  distance: 2,
                  isImmutable: attr.immutable,
                });
              }
            }
          });
        });
      });

      // Find affected functionalities
      const allDownstream = [...directDownstream, ...transitiveDownstream];
      const affectedPaths = new Set([targetPath, ...allDownstream.map(d => d.path)]);
      const affectedFunctionalities = funcs
        .filter(f => f.requiredAttributes.some(ra => affectedPaths.has(ra.abstractPath)))
        .map(f => f.id);

      // Check for immutable dependents
      const immutablePaths = allDownstream.filter(d => d.isImmutable).map(d => d.path);
      const hasImmutableDependents = immutablePaths.length > 0;

      return {
        targetPath,
        directDependencies: [...directUpstream, ...directDownstream],
        transitiveDependencies: transitiveDownstream,
        affectedRules: dependentRules,
        affectedFunctionalities,
        hasImmutableDependents,
        immutablePaths,
      };
    }
    return fetchJson(`/api/products/${productId}/impact-analysis`, {
      method: 'POST',
      body: JSON.stringify({ targetPath }),
    });
  },

  async checkModification(productId: string, targetPath: string): Promise<{
    canModify: boolean;
    reason?: string;
    requiresClone: boolean;
    affectedImmutablePaths: string[];
  }> {
    const impact = await this.analyze(productId, targetPath);

    // Check if target itself is immutable
    const targetAttr = mockAbstractAttributes.find(a => a.abstractPath === targetPath);
    if (targetAttr?.immutable) {
      return {
        canModify: false,
        reason: 'This attribute is marked as immutable and cannot be modified directly.',
        requiresClone: true,
        affectedImmutablePaths: [targetPath],
      };
    }

    // Check if any downstream dependencies are immutable
    if (impact.hasImmutableDependents) {
      return {
        canModify: false,
        reason: `Modifying this attribute would affect ${impact.immutablePaths.length} immutable attribute(s).`,
        requiresClone: true,
        affectedImmutablePaths: impact.immutablePaths,
      };
    }

    return {
      canModify: true,
      requiresClone: false,
      affectedImmutablePaths: [],
    };
  },
};

// =============================================================================
// AI API
// =============================================================================

export const aiApi = {
  async generateRule(
    _productId: string,
    naturalLanguage: string
  ): Promise<{ displayExpression: string; compiledExpression: string }> {
    // In production, this would call an AI service with _productId
    // For now, return a mock response
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Simple pattern matching for demo
    if (naturalLanguage.toLowerCase().includes('multiply')) {
      return {
        displayExpression: 'result = value * factor',
        compiledExpression: JSON.stringify({ '*': [{ 'var': 'value' }, { 'var': 'factor' }] }),
      };
    }
    if (naturalLanguage.toLowerCase().includes('if') || naturalLanguage.toLowerCase().includes('when')) {
      return {
        displayExpression: 'result = IF condition THEN value1 ELSE value2',
        compiledExpression: JSON.stringify({
          'if': [[{ 'var': 'condition' }, { 'var': 'value1' }, { 'var': 'value2' }]],
        }),
      };
    }
    return {
      displayExpression: naturalLanguage,
      compiledExpression: JSON.stringify({ 'var': 'input' }),
    };
  },

  async explainRule(compiledExpression: string): Promise<string> {
    await new Promise(resolve => setTimeout(resolve, 500));
    try {
      const expr = JSON.parse(compiledExpression);
      return explainJsonLogic(expr);
    } catch {
      return 'Unable to parse rule expression';
    }
  },

  async suggestOptimizations(_productId: string): Promise<string[]> {
    await new Promise(resolve => setTimeout(resolve, 800));
    return [
      'Consider combining age_factor and vehicle_factor calculations into a single rule',
      'The discount_rate rule could be expanded to support multiple cover types',
      'Add validation rules to ensure customer_age is within valid range (18-100)',
    ];
  },
};

function explainJsonLogic(expr: unknown, depth = 0): string {
  if (expr === null || expr === undefined) return 'nothing';
  if (typeof expr !== 'object') return String(expr);
  if (Array.isArray(expr)) return expr.map(e => explainJsonLogic(e, depth)).join(', ');

  const obj = expr as Record<string, unknown>;
  const operator = Object.keys(obj)[0];
  const operands = obj[operator];

  switch (operator) {
    case 'var':
      return `the value of "${operands}"`;
    case '+':
      return `add ${explainArray(operands)}`;
    case '-':
      return `subtract ${explainArray(operands)}`;
    case '*':
      return `multiply ${explainArray(operands)}`;
    case '/':
      return `divide ${explainArray(operands)}`;
    case '<':
      return `${explainJsonLogic((operands as unknown[])[0])} is less than ${explainJsonLogic((operands as unknown[])[1])}`;
    case '>':
      return `${explainJsonLogic((operands as unknown[])[0])} is greater than ${explainJsonLogic((operands as unknown[])[1])}`;
    case '==':
      return `${explainJsonLogic((operands as unknown[])[0])} equals ${explainJsonLogic((operands as unknown[])[1])}`;
    case 'if':
      return 'a conditional expression';
    default:
      return `${operator} operation`;
  }
}

function explainArray(arr: unknown): string {
  if (!Array.isArray(arr)) return explainJsonLogic(arr);
  return arr.map(e => explainJsonLogic(e)).join(' and ');
}

// =============================================================================
// EXPORT ALL
// =============================================================================

export const api = {
  products: productApi,
  abstractAttributes: abstractAttributeApi,
  attributes: attributeApi,
  rules: ruleApi,
  functionalities: functionalityApi,
  datatypes: datatypeApi,
  templates: templateApi,
  evaluation: evaluationApi,
  impact: impactApi,
  ai: aiApi,
};

export default api;
