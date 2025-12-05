// Enhanced AI Chat with Rule Generation
// Natural language to JSON Logic conversion, explanations, and suggestions

import { useState, useRef, useEffect, useCallback } from 'react';
import { useProductStore, useUIStore } from '@/store';
import { api } from '@/services/api';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { cn } from '@/lib/utils';
import {
  Send,
  Bot,
  User,
  Loader2,
  Sparkles,
  Copy,
  Check,
  Plus,
  Play,
  Eye,
  GitBranch,
  Lightbulb,
  ChevronDown,
  ChevronRight,
  Wand2,
  Target,
  Zap,
} from 'lucide-react';

// =============================================================================
// TYPES
// =============================================================================

interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  ruleGenerated?: {
    expression: Record<string, unknown>;
    displayExpression: string;
    inputs: string[];
    outputs: string[];
  };
  suggestions?: string[];
  impactAnalysis?: {
    affectedAttributes: string[];
    affectedRules: string[];
  };
}

interface QuickAction {
  id: string;
  label: string;
  icon: React.ElementType;
  prompt: string;
  color: string;
}

// =============================================================================
// QUICK ACTIONS
// =============================================================================

const quickActions: QuickAction[] = [
  {
    id: 'create-rule',
    label: 'Create Rule',
    icon: Plus,
    prompt: 'Create a rule that ',
    color: 'bg-emerald-100 text-emerald-700 border-emerald-200',
  },
  {
    id: 'explain',
    label: 'Explain Rule',
    icon: Eye,
    prompt: 'Explain how the ',
    color: 'bg-blue-100 text-blue-700 border-blue-200',
  },
  {
    id: 'optimize',
    label: 'Optimize',
    icon: Zap,
    prompt: 'Suggest optimizations for my rules',
    color: 'bg-amber-100 text-amber-700 border-amber-200',
  },
  {
    id: 'impact',
    label: 'Impact Analysis',
    icon: Target,
    prompt: 'What would be affected if I change ',
    color: 'bg-purple-100 text-purple-700 border-purple-200',
  },
];

const suggestedPrompts = [
  'Create a rule: if age > 60, add 20% to premium',
  'Explain how premium calculation works',
  'What attributes affect the final_premium?',
  'Suggest improvements for my rules',
  'Test the discount rule with quantity=10',
  'Create a validation rule for customer age between 18-100',
];

// =============================================================================
// MESSAGE COMPONENTS
// =============================================================================

interface RuleCardProps {
  rule: NonNullable<Message['ruleGenerated']>;
  onAdd: () => void;
  onTest: () => void;
}

function RuleCard({ rule, onAdd, onTest }: RuleCardProps) {
  const [copied, setCopied] = useState(false);
  const [expanded, setExpanded] = useState(true);

  const handleCopy = () => {
    navigator.clipboard.writeText(JSON.stringify(rule.expression, null, 2));
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="mt-3 rounded-lg border bg-white shadow-sm overflow-hidden">
      {/* Header */}
      <div
        className="flex items-center justify-between px-3 py-2 bg-gradient-to-r from-emerald-50 to-teal-50 cursor-pointer"
        onClick={() => setExpanded(!expanded)}
      >
        <div className="flex items-center gap-2">
          <GitBranch className="h-4 w-4 text-emerald-600" />
          <span className="text-sm font-medium text-gray-800">Generated Rule</span>
        </div>
        {expanded ? (
          <ChevronDown className="h-4 w-4 text-gray-400" />
        ) : (
          <ChevronRight className="h-4 w-4 text-gray-400" />
        )}
      </div>

      {expanded && (
        <div className="p-3 space-y-3">
          {/* Display Expression */}
          <div className="rounded-md bg-emerald-50 px-3 py-2 border border-emerald-100">
            <p className="text-sm font-medium text-emerald-800">{rule.displayExpression}</p>
          </div>

          {/* JSON Logic */}
          <div className="relative">
            <pre className="rounded-md bg-gray-900 p-3 text-xs font-mono text-gray-100 overflow-auto max-h-[150px]">
              {JSON.stringify(rule.expression, null, 2)}
            </pre>
            <button
              onClick={handleCopy}
              className="absolute top-2 right-2 p-1.5 rounded bg-gray-800 hover:bg-gray-700 text-gray-300"
            >
              {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
            </button>
          </div>

          {/* Inputs/Outputs */}
          <div className="flex gap-4 text-xs">
            <div>
              <span className="text-gray-500">Inputs:</span>{' '}
              <span className="font-medium">{rule.inputs.join(', ') || 'None'}</span>
            </div>
            <div>
              <span className="text-gray-500">Outputs:</span>{' '}
              <span className="font-medium">{rule.outputs.join(', ') || 'Result'}</span>
            </div>
          </div>

          {/* Actions */}
          <div className="flex gap-2 pt-2 border-t">
            <Button size="sm" onClick={onAdd} className="gap-1.5 flex-1">
              <Plus className="h-3.5 w-3.5" />
              Add to Rules
            </Button>
            <Button size="sm" variant="outline" onClick={onTest} className="gap-1.5 flex-1">
              <Play className="h-3.5 w-3.5" />
              Test Rule
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}

interface SuggestionsCardProps {
  suggestions: string[];
  onSelect: (suggestion: string) => void;
}

function SuggestionsCard({ suggestions, onSelect }: SuggestionsCardProps) {
  return (
    <div className="mt-3 rounded-lg border bg-amber-50 p-3">
      <div className="flex items-center gap-2 mb-2">
        <Lightbulb className="h-4 w-4 text-amber-600" />
        <span className="text-sm font-medium text-amber-800">Suggestions</span>
      </div>
      <ul className="space-y-1.5">
        {suggestions.map((suggestion, i) => (
          <li key={i}>
            <button
              onClick={() => onSelect(suggestion)}
              className="text-left text-sm text-amber-700 hover:text-amber-900 hover:underline"
            >
              {suggestion}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}

// =============================================================================
// MAIN COMPONENT
// =============================================================================

export function AIChat() {
  const { selectedProduct, abstractAttributes, rules, createRule } = useProductStore();
  const { impactAnalysisTarget: _unused, setImpactAnalysisTarget: _setImpact } = useUIStore();
  // Keeping for future impact analysis integration
  void _unused;
  void _setImpact;

  const [messages, setMessages] = useState<Message[]>([
    {
      id: '1',
      role: 'assistant',
      content: `Hello! I'm your AI assistant for Product-FARM. I can help you:\n\n**Create rules** - Describe what you want in plain English\n**Explain rules** - Understand complex business logic\n**Analyze impact** - See what changes affect\n**Optimize** - Get suggestions for improvement\n\nWhat would you like to do?`,
      timestamp: new Date(),
    },
  ]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  // Process user message and generate response
  const processMessage = useCallback(
    async (userInput: string) => {
      const lowerInput = userInput.toLowerCase();
      let response: Message;

      // Simulate AI processing delay
      await new Promise((resolve) => setTimeout(resolve, 1000 + Math.random() * 1000));

      // Rule creation intent
      if (
        lowerInput.includes('create') &&
        (lowerInput.includes('rule') || lowerInput.includes('calculate'))
      ) {
        try {
          const generated = await api.ai.generateRule(selectedProduct?.id || '', userInput);

          // Parse inputs from expression
          const inputs: string[] = [];
          const expressionStr = JSON.stringify(generated.compiledExpression);
          const varMatches = expressionStr.match(/"var"\s*:\s*"([^"]+)"/g);
          varMatches?.forEach((match) => {
            const varName = match.match(/"var"\s*:\s*"([^"]+)"/)?.[1];
            if (varName && !inputs.includes(varName)) {
              inputs.push(varName);
            }
          });

          response = {
            id: crypto.randomUUID(),
            role: 'assistant',
            content: "I've created a rule based on your description. Here's what I generated:",
            timestamp: new Date(),
            ruleGenerated: {
              expression: JSON.parse(generated.compiledExpression),
              displayExpression: generated.displayExpression,
              inputs,
              outputs: ['result'],
            },
          };
        } catch {
          response = {
            id: crypto.randomUUID(),
            role: 'assistant',
            content:
              "I had trouble generating that rule. Could you try rephrasing it? For example:\n\n- 'Create a rule: if age > 60, multiply premium by 1.2'\n- 'Calculate discount as 10% when order_value > 1000'",
            timestamp: new Date(),
          };
        }
      }
      // Explain intent
      else if (lowerInput.includes('explain') || lowerInput.includes('how does')) {
        const ruleNames = rules.map((r) => r.description || r.displayExpression.substring(0, 30));
        const matchingRule = rules.find(
          (r) =>
            lowerInput.includes(r.displayExpression.toLowerCase().substring(0, 20)) ||
            (r.description && lowerInput.includes(r.description.toLowerCase()))
        );

        if (matchingRule) {
          const explanation = await api.ai.explainRule(matchingRule.compiledExpression);
          response = {
            id: crypto.randomUUID(),
            role: 'assistant',
            content: `**Rule Explanation:**\n\n${matchingRule.displayExpression}\n\n${explanation}\n\n**Inputs:** ${matchingRule.inputAttributes.map((a) => a.attributePath.split(':').pop()).join(', ')}\n**Outputs:** ${matchingRule.outputAttributes.map((a) => a.attributePath.split(':').pop()).join(', ')}`,
            timestamp: new Date(),
          };
        } else {
          response = {
            id: crypto.randomUUID(),
            role: 'assistant',
            content: `I can explain any of these rules:\n\n${ruleNames.map((n, i) => `${i + 1}. ${n}`).join('\n')}\n\nWhich one would you like me to explain?`,
            timestamp: new Date(),
          };
        }
      }
      // Impact analysis
      else if (lowerInput.includes('impact') || lowerInput.includes('affect')) {
        const attrNames = abstractAttributes.map((a) => a.attributeName || a.abstractPath.split(':').pop());
        const mentionedAttr = attrNames.find((name) => name && lowerInput.includes(name.toLowerCase()));

        if (mentionedAttr) {
          const affectedRules = rules.filter((r) =>
            r.inputAttributes.some((i) => i.attributePath.includes(mentionedAttr))
          );
          const affectedOutputs = affectedRules.flatMap((r) =>
            r.outputAttributes.map((o) => o.attributePath.split(':').pop())
          );

          response = {
            id: crypto.randomUUID(),
            role: 'assistant',
            content: `**Impact Analysis for \`${mentionedAttr}\`:**\n\n**Directly affects ${affectedRules.length} rules:**\n${affectedRules.map((r) => `- ${r.description || r.displayExpression.substring(0, 40)}`).join('\n')}\n\n**Computed attributes that will change:**\n${[...new Set(affectedOutputs)].map((o) => `- ${o}`).join('\n') || 'None'}\n\n*Click the target icon on any attribute in the graph to visualize the impact.*`,
            timestamp: new Date(),
            impactAnalysis: {
              affectedAttributes: [...new Set(affectedOutputs)] as string[],
              affectedRules: affectedRules.map((r) => r.id),
            },
          };
        } else {
          response = {
            id: crypto.randomUUID(),
            role: 'assistant',
            content:
              'Which attribute would you like to analyze? You can ask something like:\n\n- "What would be affected if I change customer_age?"\n- "Show impact of modifying vehicle_value"',
            timestamp: new Date(),
          };
        }
      }
      // Optimization suggestions
      else if (
        lowerInput.includes('optimize') ||
        lowerInput.includes('improve') ||
        lowerInput.includes('suggest')
      ) {
        const suggestions = await api.ai.suggestOptimizations(selectedProduct?.id || '');

        response = {
          id: crypto.randomUUID(),
          role: 'assistant',
          content: 'Here are some suggestions to improve your rules:',
          timestamp: new Date(),
          suggestions,
        };
      }
      // Test intent
      else if (lowerInput.includes('test')) {
        response = {
          id: crypto.randomUUID(),
          role: 'assistant',
          content:
            'To test your rules, use the **Simulation Panel** on the right side of the screen. You can:\n\n1. Enter input values for each attribute\n2. Click "Run" or enable "Auto" mode\n3. See computed outputs in real-time\n4. Save test scenarios for later\n\n*The simulation panel updates automatically as you change inputs when auto-mode is enabled.*',
          timestamp: new Date(),
        };
      }
      // Default response
      else {
        response = {
          id: crypto.randomUUID(),
          role: 'assistant',
          content: `I understand you're asking about: "${userInput}"\n\nI can help you with:\n- **Creating rules**: "Create a rule that calculates discount based on quantity"\n- **Explaining rules**: "Explain the premium calculation rule"\n- **Impact analysis**: "What would be affected if I change customer_age?"\n- **Optimizing**: "Suggest improvements for my rules"`,
          timestamp: new Date(),
        };
      }

      return response;
    },
    [selectedProduct, abstractAttributes, rules]
  );

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    const userMessage: Message = {
      id: crypto.randomUUID(),
      role: 'user',
      content: input,
      timestamp: new Date(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInput('');
    setIsLoading(true);

    try {
      const response = await processMessage(input);
      setMessages((prev) => [...prev, response]);
    } catch (error) {
      setMessages((prev) => [
        ...prev,
        {
          id: crypto.randomUUID(),
          role: 'assistant',
          content: 'Sorry, I encountered an error processing your request. Please try again.',
          timestamp: new Date(),
        },
      ]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleQuickAction = (action: QuickAction) => {
    setInput(action.prompt);
  };

  const handleSuggestedPrompt = (prompt: string) => {
    setInput(prompt);
  };

  const handleAddRule = useCallback(
    async (rule: NonNullable<Message['ruleGenerated']>) => {
      try {
        await createRule({
          productId: selectedProduct?.id,
          ruleType: 'calculation',
          displayExpression: rule.displayExpression,
          compiledExpression: JSON.stringify(rule.expression),
          enabled: true,
        });

        setMessages((prev) => [
          ...prev,
          {
            id: crypto.randomUUID(),
            role: 'system',
            content: 'Rule added successfully to your product.',
            timestamp: new Date(),
          },
        ]);
      } catch (error) {
        setMessages((prev) => [
          ...prev,
          {
            id: crypto.randomUUID(),
            role: 'system',
            content: 'Failed to add rule. Please try again.',
            timestamp: new Date(),
          },
        ]);
      }
    },
    [createRule, selectedProduct]
  );

  const handleTestRule = useCallback((_rule: NonNullable<Message['ruleGenerated']>) => {
    // This would open the simulation panel with the rule loaded
    // Future: actually load _rule into simulation panel
    setMessages((prev) => [
      ...prev,
      {
        id: crypto.randomUUID(),
        role: 'system',
        content: 'Open the Simulation Panel to test this rule with your inputs.',
        timestamp: new Date(),
      },
    ]);
  }, []);

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex-shrink-0 border-b px-4 py-3">
        <div className="flex items-center gap-2">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-gradient-to-br from-indigo-500 to-purple-600">
            <Wand2 className="h-4 w-4 text-white" />
          </div>
          <div>
            <h3 className="text-sm font-semibold text-gray-800">AI Assistant</h3>
            <p className="text-[10px] text-gray-500">Powered by Product-FARM AI</p>
          </div>
        </div>
      </div>

      {/* Quick Actions */}
      <div className="flex-shrink-0 border-b p-2">
        <div className="flex gap-1.5 overflow-x-auto pb-1">
          {quickActions.map((action) => (
            <button
              key={action.id}
              onClick={() => handleQuickAction(action)}
              className={cn(
                'flex items-center gap-1.5 whitespace-nowrap rounded-full border px-2.5 py-1 text-xs font-medium transition-colors',
                action.color
              )}
            >
              <action.icon className="h-3 w-3" />
              {action.label}
            </button>
          ))}
        </div>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-auto p-4 space-y-4">
        {messages.map((message) => (
          <div
            key={message.id}
            className={cn(
              'flex gap-3',
              message.role === 'user' ? 'flex-row-reverse' : 'flex-row',
              message.role === 'system' && 'justify-center'
            )}
          >
            {message.role !== 'system' && (
              <div
                className={cn(
                  'flex h-8 w-8 shrink-0 items-center justify-center rounded-full',
                  message.role === 'user'
                    ? 'bg-primary'
                    : 'bg-gradient-to-br from-indigo-500 to-purple-600'
                )}
              >
                {message.role === 'user' ? (
                  <User className="h-4 w-4 text-primary-foreground" />
                ) : (
                  <Bot className="h-4 w-4 text-white" />
                )}
              </div>
            )}

            <div
              className={cn(
                'rounded-lg px-4 py-2 max-w-[85%]',
                message.role === 'user'
                  ? 'bg-primary text-primary-foreground'
                  : message.role === 'system'
                    ? 'bg-gray-100 text-gray-600 text-xs'
                    : 'bg-muted'
              )}
            >
              <div className="whitespace-pre-wrap text-sm prose prose-sm max-w-none">
                {message.content.split('\n').map((line, i) => {
                  // Handle bold text
                  const parts = line.split(/(\*\*[^*]+\*\*)/g);
                  return (
                    <p key={i} className={i > 0 ? 'mt-1' : ''}>
                      {parts.map((part, j) => {
                        if (part.startsWith('**') && part.endsWith('**')) {
                          return (
                            <strong key={j} className="font-semibold">
                              {part.slice(2, -2)}
                            </strong>
                          );
                        }
                        // Handle inline code
                        const codeParts = part.split(/(`[^`]+`)/g);
                        return codeParts.map((codePart, k) => {
                          if (codePart.startsWith('`') && codePart.endsWith('`')) {
                            return (
                              <code
                                key={`${j}-${k}`}
                                className="rounded bg-gray-200 px-1 py-0.5 text-xs font-mono"
                              >
                                {codePart.slice(1, -1)}
                              </code>
                            );
                          }
                          return codePart;
                        });
                      })}
                    </p>
                  );
                })}
              </div>

              {/* Rule Card */}
              {message.ruleGenerated && (
                <RuleCard
                  rule={message.ruleGenerated}
                  onAdd={() => handleAddRule(message.ruleGenerated!)}
                  onTest={() => handleTestRule(message.ruleGenerated!)}
                />
              )}

              {/* Suggestions */}
              {message.suggestions && (
                <SuggestionsCard
                  suggestions={message.suggestions}
                  onSelect={handleSuggestedPrompt}
                />
              )}
            </div>
          </div>
        ))}

        {isLoading && (
          <div className="flex gap-3">
            <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-indigo-500 to-purple-600">
              <Bot className="h-4 w-4 text-white" />
            </div>
            <div className="rounded-lg bg-muted px-4 py-2">
              <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Suggested Prompts (show only initially) */}
      {messages.length <= 1 && (
        <div className="flex-shrink-0 border-t p-3">
          <p className="mb-2 text-xs font-medium text-muted-foreground flex items-center gap-1">
            <Sparkles className="h-3 w-3" />
            Try these:
          </p>
          <div className="flex flex-wrap gap-2">
            {suggestedPrompts.slice(0, 4).map((prompt) => (
              <button
                key={prompt}
                onClick={() => handleSuggestedPrompt(prompt)}
                className="rounded-full border bg-background px-3 py-1 text-xs text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
              >
                {prompt.length > 35 ? prompt.substring(0, 35) + '...' : prompt}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Input */}
      <div className="flex-shrink-0 border-t p-4">
        <form onSubmit={handleSubmit} className="flex gap-2">
          <Input
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Ask about rules, create new ones, or get suggestions..."
            disabled={isLoading}
            className="flex-1"
          />
          <Button type="submit" size="icon" disabled={isLoading || !input.trim()}>
            <Send className="h-4 w-4" />
          </Button>
        </form>
      </div>
    </div>
  );
}

export default AIChat;
