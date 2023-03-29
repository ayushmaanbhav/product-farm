package io.github.ayushmaanbhav.rule.domain.ruleEngine

import io.github.ayushmaanbhav.rule.domain.ruleEngine.algorithm.RuleDependencyGraphBuilder
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.Query
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.RuleEngineConfig
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryContext
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryIdentifier
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryInput
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.QueryOutput
import org.apache.logging.log4j.kotlin.Logging

class CacheEnabledRuleEngine(config: RuleEngineConfig) : Logging {
    private val cache = RuleEngineCache(config)
    private val evaluator = JsonLogicEvaluator(config)

    fun evaluate(context: QueryContext, queries: List<Query>, input: QueryInput): QueryOutput {
        val rules = cache.get(
            context.identifier, QueryIdentifier(context.identifier, queries),
            { buildRuleDependencyGraph(context) }, { rdg -> rdg.computeExecutableRules(queries) }
        )
        return QueryOutput(evaluator.evaluate(rules, input.attributes))
    }

    private fun buildRuleDependencyGraph(context: QueryContext): RuleDependencyGraph {
        val ruleDependencyGraphBuilder = RuleDependencyGraphBuilder()
        context.rules.forEach(ruleDependencyGraphBuilder::visit)
        return ruleDependencyGraphBuilder.build()
    }
}
