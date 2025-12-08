// Product Farm Type Definitions
// Aligned with backend gRPC API

// =============================================================================
// ENUMS AND CONSTANTS (matching backend proto definitions)
// =============================================================================

/**
 * Primitive types supported by the datatype system
 * Maps to backend PrimitiveType enum
 */
export type PrimitiveType =
  | 'STRING'
  | 'INT'
  | 'FLOAT'
  | 'DECIMAL'
  | 'BOOL'
  | 'DATETIME'
  | 'ENUM'
  | 'ARRAY'
  | 'OBJECT';

export const PRIMITIVE_TYPES: PrimitiveType[] = [
  'STRING',
  'INT',
  'FLOAT',
  'DECIMAL',
  'BOOL',
  'DATETIME',
  'ENUM',
  'ARRAY',
  'OBJECT',
];

/**
 * How an attribute gets its value
 * Maps to backend AttributeValueType enum
 */
export type AttributeValueType =
  | 'FIXED_VALUE'    // Value is directly set
  | 'RULE_DRIVEN'    // Value computed by a rule
  | 'JUST_DEFINITION'; // No value, just schema definition

export const ATTRIBUTE_VALUE_TYPES: AttributeValueType[] = [
  'FIXED_VALUE',
  'RULE_DRIVEN',
  'JUST_DEFINITION',
];

/**
 * Display name format types
 * Maps to backend DisplayNameFormat enum
 */
export type DisplayNameFormat =
  | 'SYSTEM'    // System-generated format
  | 'HUMAN'     // Human-readable format
  | 'ORIGINAL'; // Original input format

export const DISPLAY_NAME_FORMATS: DisplayNameFormat[] = [
  'SYSTEM',
  'HUMAN',
  'ORIGINAL',
];

/**
 * Types of relationships between attributes
 * Maps to backend AttributeRelationshipType enum
 */
export type AttributeRelationshipType =
  | 'ENUMERATION'       // Related to an enumeration
  | 'KEY_ENUMERATION'   // Acts as key in key-value enumeration
  | 'VALUE_ENUMERATION'; // Acts as value in key-value enumeration

export const ATTRIBUTE_RELATIONSHIP_TYPES: AttributeRelationshipType[] = [
  'ENUMERATION',
  'KEY_ENUMERATION',
  'VALUE_ENUMERATION',
];

/**
 * Constraints that can be applied to a datatype
 * Used for validation at runtime
 */
export interface DatatypeConstraints {
  // Numeric constraints
  min?: number;
  max?: number;

  // String constraints
  minLength?: number;
  maxLength?: number;
  pattern?: string; // Regex pattern

  // Decimal constraints
  precision?: number; // Total digits
  scale?: number;     // Digits after decimal

  // Array constraints
  minItems?: number;
  maxItems?: number;
  uniqueItems?: boolean;

  // Enum constraints (for validation)
  allowedValues?: string[];

  // Constraint rule support (JSON Logic expression)
  constraintRuleExpression?: string; // JSON Logic expression that must return true
  constraintErrorMessage?: string;   // Custom error message when validation fails
}

/**
 * Display name with format information
 */
export interface DisplayName {
  name: string;
  format: DisplayNameFormat;
  orderIndex: number;
}

// =============================================================================
// CORE ENTITIES
// =============================================================================

export interface Product {
  id: string;
  name: string;
  description: string;
  templateType: string;
  status: ProductStatus;
  parentProductId?: string;
  effectiveFrom: number; // Unix timestamp
  expiryAt?: number;
  createdAt: number;
  updatedAt: number;
  version: number;
}

export type ProductStatus =
  | 'DRAFT'
  | 'PENDING_APPROVAL'
  | 'ACTIVE'
  | 'DISCONTINUED';

export interface AbstractAttribute {
  abstractPath: string;
  productId: string;
  componentType: string;
  componentId?: string;
  attributeName: string;
  datatypeId: string;
  enumName?: string; // For enum types
  description: string;
  constraintExpression?: string; // JSON string for additional constraints
  displayExpression: string;
  displayNames: DisplayName[]; // Multiple display name formats
  tags: AttributeTag[];
  relatedAttributes: RelatedAttribute[];
  immutable: boolean; // Cannot be modified once set
  createdAt?: number;
  updatedAt?: number;
}

export interface AttributeTag {
  name: string;
  orderIndex: number;
}

export interface RelatedAttribute {
  relationshipType: AttributeRelationshipType;
  relatedPath: string;
  orderIndex: number;
}

export interface Attribute {
  path: string;
  abstractPath: string;
  productId: string;
  componentType: string;
  componentId: string;
  attributeName: string;
  valueType: AttributeValueType;
  value?: AttributeValue;
  ruleId?: string; // Set when valueType is RULE_DRIVEN
  createdAt: number;
  updatedAt: number;
}

export interface Rule {
  id: string;
  productId: string;
  ruleType: string;
  displayExpression: string;
  compiledExpression: string; // JSON Logic expression as string
  description?: string;
  enabled: boolean;
  orderIndex: number;
  inputAttributes: RuleInputAttribute[];
  outputAttributes: RuleOutputAttribute[];
  createdAt: number;
  updatedAt: number;
}

export interface RuleInputAttribute {
  ruleId: string;
  attributePath: string;
  orderIndex: number;
}

export interface RuleOutputAttribute {
  ruleId: string;
  attributePath: string;
  orderIndex: number;
}

export interface DataType {
  id: string;
  name: string;
  primitiveType: PrimitiveType;
  constraints: DatatypeConstraints;
  constraintsJson?: string; // Raw JSON string for custom constraints
  description?: string;
  createdAt?: number;
  updatedAt?: number;
}

// Legacy alias for backwards compatibility
export type { DataType as Datatype };

export interface ProductFunctionality {
  id: string;
  productId: string;
  name: string;
  displayName: string;
  description: string;
  status: FunctionalityStatus;
  immutable: boolean; // Cannot be modified once set
  requiredAttributes: FunctionalityRequiredAttribute[];
  createdAt: number;
  updatedAt: number;
}

export type FunctionalityStatus =
  | 'DRAFT'
  | 'PENDING_APPROVAL'
  | 'ACTIVE'; // Changed from APPROVED to match backend

export interface FunctionalityRequiredAttribute {
  functionalityId: string;
  abstractPath: string;
  description: string;
  orderIndex: number;
}

export interface TemplateEnumeration {
  id: string;
  name: string;
  templateType: string;
  values: string[];
  description?: string;
}

// =============================================================================
// VALUE TYPES
// =============================================================================

export type AttributeValue =
  | { type: 'null' }
  | { type: 'bool'; value: boolean }
  | { type: 'int'; value: number }
  | { type: 'float'; value: number }
  | { type: 'string'; value: string }
  | { type: 'decimal'; value: string }
  | { type: 'array'; value: AttributeValue[] }
  | { type: 'object'; value: Record<string, AttributeValue> };

// =============================================================================
// EVALUATION TYPES
// =============================================================================

export interface EvaluateRequest {
  productId: string;
  inputData: Record<string, AttributeValue>;
  ruleIds?: string[];
  options?: EvaluationOptions;
}

export interface EvaluationOptions {
  includeIntermediateResults: boolean;
  maxExecutionTimeMs: number;
  debugMode: boolean;
}

export interface EvaluateResponse {
  success: boolean;
  outputs: Record<string, AttributeValue>;
  ruleResults: RuleResult[];
  metrics: ExecutionMetrics;
  errors: EvaluationError[];
}

export interface RuleResult {
  ruleId: string;
  outputs: OutputValue[];
  executionTimeNs: number;
  skipped: boolean;
  skipReason?: string;
  error?: string;
}

export interface OutputValue {
  path: string;
  value: AttributeValue;
}

export interface ExecutionMetrics {
  totalTimeNs: number;
  rulesExecuted: number;
  rulesSkipped: number;
  cacheHits: number;
  levels: LevelMetrics[];
}

export interface LevelMetrics {
  level: number;
  timeNs: number;
  rulesCount: number;
}

export interface EvaluationError {
  attribute: string;
  message: string;
  ruleId?: string;
}

// =============================================================================
// GRAPH VISUALIZATION TYPES
// =============================================================================

export interface GraphNode {
  id: string;
  type: 'input' | 'computed' | 'rule' | 'functionality';
  label: string;
  position: { x: number; y: number };
  data: GraphNodeData;
}

export interface GraphNodeData {
  path?: string;
  ruleType?: string;
  datatype?: string;
  description?: string;
  enabled?: boolean;
  tags?: string[];
  inputCount?: number;
  outputCount?: number;
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  type: 'input' | 'output';
  animated?: boolean;
  label?: string;
}

export interface ExecutionPlan {
  levels: ExecutionLevel[];
  dependencies: RuleDependency[];
  missingInputs: MissingInput[];
  hasCycles: boolean;
}

export interface ExecutionLevel {
  level: number;
  ruleIds: string[];
}

export interface RuleDependency {
  ruleId: string;
  dependsOn: string[];
}

export interface MissingInput {
  ruleId: string;
  inputPath: string;
}

// =============================================================================
// UI STATE TYPES
// =============================================================================

export interface SimulationScenario {
  id: string;
  name: string;
  inputs: Record<string, unknown>;
  expectedOutputs?: Record<string, unknown>;
}

export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export interface ValidationError {
  code: string;
  message: string;
  path?: string;
  ruleId?: string;
}

export interface ValidationWarning {
  code: string;
  message: string;
  path?: string;
  ruleId?: string;
}

// =============================================================================
// JSON LOGIC BLOCK TYPES (for visual editor)
// =============================================================================

export type JsonLogicBlock =
  | VariableBlock
  | LiteralBlock
  | OperatorBlock
  | ConditionBlock
  | ArrayBlock;

export interface VariableBlock {
  type: 'variable';
  path: string;
  datatype?: string;
}

export interface LiteralBlock {
  type: 'literal';
  value: unknown;
  datatype: string;
}

export interface OperatorBlock {
  type: 'operator';
  operator: string;
  operands: JsonLogicBlock[];
}

export interface ConditionBlock {
  type: 'condition';
  conditions: Array<{
    if: JsonLogicBlock;
    then: JsonLogicBlock;
  }>;
  else?: JsonLogicBlock;
}

export interface ArrayBlock {
  type: 'array';
  operation: 'map' | 'filter' | 'reduce' | 'some' | 'all' | 'none';
  array: JsonLogicBlock;
  lambda: JsonLogicBlock;
  initial?: JsonLogicBlock;
}

// =============================================================================
// API RESPONSE TYPES
// =============================================================================

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  nextPageToken: string;
  totalCount: number;
}

// =============================================================================
// UTILITY TYPES
// =============================================================================

export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export interface Position {
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface Bounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

// =============================================================================
// CLONE WORKFLOW TYPES
// =============================================================================

export interface CloneProductRequest {
  sourceProductId: string;
  newProductId: string; // Required - must pass validation
  newProductName: string;
  newProductDescription?: string;
  newEffectiveFrom?: number;
  // Optional selections for wizard-based cloning
  selectedComponents?: string[];
  selectedDatatypes?: string[];
  selectedEnumerations?: string[];
  selectedFunctionalities?: string[];
  selectedAbstractAttributes?: string[];
  cloneConcreteAttributes?: boolean; // Whether to clone concrete attribute values
}

export interface CloneProductResponse {
  newProductId: string;
  abstractAttributesCloned: number;
  attributesCloned: number;
  rulesCloned: number;
  functionalitiesCloned: number;
  pathMapping: Record<string, string>; // Maps old paths to new paths
}

// =============================================================================
// FUNCTIONALITY EVALUATION TYPES
// =============================================================================

export interface EvaluateFunctionalityRequest {
  productId: string;
  functionalityId: string;
  inputData: Record<string, AttributeValue>;
  options?: EvaluationOptions;
}

export interface EvaluateFunctionalityResponse {
  success: boolean;
  functionalityId: string;
  functionalityName: string;
  outputs: Record<string, AttributeValue>;
  missingRequiredAttributes: string[];
  metrics: ExecutionMetrics;
  errors: EvaluationError[];
}

// =============================================================================
// APPROVAL WORKFLOW TYPES
// =============================================================================

export interface ProductApproval {
  id: string;
  productId: string;
  status: ApprovalStatus;
  submittedBy: string;
  submittedAt: number;
  reviewedBy?: string;
  reviewedAt?: number;
  comments?: string;
}

export type ApprovalStatus =
  | 'PENDING'
  | 'APPROVED'
  | 'REJECTED';

export interface SubmitForApprovalRequest {
  productId: string;
  comments?: string;
}

export interface ApprovalDecisionRequest {
  productId: string;
  approved: boolean;
  comments?: string;
}

export interface FunctionalityApprovalRequest {
  productId: string;
  functionalityId: string;
  approved: boolean;
  comments?: string;
}

// =============================================================================
// IMPACT ANALYSIS TYPES
// =============================================================================

export interface ImpactAnalysis {
  targetPath: string;
  directDependencies: DependencyInfo[];
  transitiveDependencies: DependencyInfo[];
  affectedRules: string[];
  affectedFunctionalities: string[];
  hasImmutableDependents: boolean;
  immutablePaths: string[];
}

export interface DependencyInfo {
  path: string;
  attributeName: string;
  direction: 'upstream' | 'downstream';
  distance: number;
  isImmutable: boolean;
}

// =============================================================================
// TEMPLATE TYPES
// =============================================================================

/**
 * Product Template - defines a reusable blueprint for creating products
 * Templates own components, datatypes, enumerations, functionalities, and abstract attributes
 */
export interface ProductTemplate {
  id: string;
  name: string;
  description: string;
  type: string;
  version: string;
  enumerations: TemplateEnumeration[];
  components: TemplateComponent[];
  datatypes: TemplateDatatypeRef[];
  functionalities: TemplateFunctionality[];
  abstractAttributes: TemplateAbstractAttribute[];
  createdAt: number;
  updatedAt: number;
}

/**
 * Template Component - a logical grouping within the template
 */
export interface TemplateComponent {
  id: string;
  name: string;
  displayName: string;
  description: string;
  isRequired: boolean; // If true, cannot be deselected during product creation
  orderIndex: number;
}

/**
 * Reference to a datatype used by the template
 */
export interface TemplateDatatypeRef {
  datatypeId: string;
  isRequired: boolean; // If true, must be included when using this template
  defaultIncluded: boolean; // If true, selected by default in wizard
}

/**
 * Functionality defined in the template
 */
export interface TemplateFunctionality {
  id: string;
  name: string;
  displayName: string;
  description: string;
  isRequired: boolean;
  defaultIncluded: boolean;
  requiredComponents: string[]; // Component IDs this functionality depends on
  requiredAbstractAttributes: string[]; // Abstract attribute names required
}

/**
 * Abstract attribute defined in the template
 */
export interface TemplateAbstractAttribute {
  name: string;
  componentId: string;
  datatypeId: string;
  enumName?: string;
  description: string;
  isInput: boolean;
  isRequired: boolean;
  defaultIncluded: boolean;
  immutable: boolean;
  tags: string[];
}

export interface CreateTemplateEnumerationRequest {
  templateType: string;
  name: string;
  values: string[];
  description?: string;
}

/**
 * Wizard selection state - tracks what user has selected during product creation
 */
export interface ProductCreationWizardState {
  step: number;
  templateId: string | null;
  productInfo: {
    id: string;
    name: string;
    description: string;
    effectiveFrom: number;
    expiryAt?: number;
  };
  selectedComponents: Set<string>;
  selectedDatatypes: Set<string>;
  selectedEnumerations: Set<string>;
  selectedFunctionalities: Set<string>;
  selectedAbstractAttributes: Set<string>;
}

// =============================================================================
// BULK OPERATION TYPES
// =============================================================================

export interface BatchEvaluateRequest {
  requests: EvaluateRequest[];
}

export interface BatchEvaluateResponse {
  results: EvaluateResponse[];
  metrics: BatchMetrics;
}

export interface BatchMetrics {
  totalRequests: number;
  successCount: number;
  failureCount: number;
  totalTimeMs: number;
  avgTimePerRequest: number;
}

// =============================================================================
// CREATE/UPDATE REQUEST TYPES
// =============================================================================

export interface CreateProductRequest {
  id: string;
  name: string;
  description: string;
  templateType: string;
  effectiveFrom: number;
  expiryAt?: number;
}

export interface CreateAbstractAttributeRequest {
  productId: string;
  componentType: string;
  componentId?: string;
  attributeName: string;
  datatypeId: string;
  enumName?: string;
  description: string;
  constraintExpression?: string;
  tags: Array<{ name: string; orderIndex: number }>;
  displayNames: Array<{ name: string; format: DisplayNameFormat; orderIndex: number }>;
  relatedAttributes?: Array<{
    relationshipType: AttributeRelationshipType;
    relatedPath: string;
    orderIndex: number;
  }>;
  immutable?: boolean;
}

export interface CreateAttributeRequest {
  productId: string;
  abstractPath: string;
  componentId: string;
  valueType: AttributeValueType;
  value?: AttributeValue;
  ruleId?: string;
}

export interface CreateDatatypeRequest {
  id: string;
  name: string;
  primitiveType: PrimitiveType;
  constraints?: DatatypeConstraints;
  description?: string;
}

export interface CreateRuleRequest {
  productId: string;
  ruleType: string;
  displayExpression: string;
  compiledExpression: string;
  description?: string;
  orderIndex?: number;
  inputAttributes: Array<{ attributePath: string; orderIndex: number }>;
  outputAttributes: Array<{ attributePath: string; orderIndex: number }>;
}

export interface CreateFunctionalityRequest {
  productId: string;
  name: string;
  displayName: string;
  description: string;
  requiredAttributes: Array<{
    abstractPath: string;
    description: string;
    orderIndex: number;
  }>;
  immutable?: boolean;
}

export interface CreateTemplateEnumerationRequest {
  templateType: string;
  name: string;
  values: string[];
  description?: string;
}

// =============================================================================
// REJECT/DISCONTINUE REQUEST TYPES
// =============================================================================

export interface RejectProductRequest {
  productId: string;
  reason: string;
  rejectorId?: string;
}

export interface DiscontinueProductRequest {
  productId: string;
  reason?: string;
}

export interface RejectFunctionalityRequest {
  productId: string;
  functionalityName: string;
  reason: string;
  rejectorId?: string;
}

// =============================================================================
// VALIDATION REQUEST/RESPONSE TYPES
// =============================================================================

export interface ValidateRulesRequest {
  productId: string;
  ruleIds?: string[]; // If empty, validates all rules
}

export interface ValidateRulesResponse {
  isValid: boolean;
  results: RuleValidationResult[];
}

export interface RuleValidationResult {
  ruleId: string;
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}
