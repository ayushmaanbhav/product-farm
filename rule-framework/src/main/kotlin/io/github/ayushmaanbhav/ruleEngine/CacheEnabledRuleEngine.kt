package io.github.ayushmaanbhav.ruleEngine

import io.github.ayushmaanbhav.ruleEngine.algorithm.DependencyGraphBuilder
import io.github.ayushmaanbhav.ruleEngine.model.Query
import io.github.ayushmaanbhav.ruleEngine.model.QueryContext
import io.github.ayushmaanbhav.ruleEngine.model.QueryIdentifier
import io.github.ayushmaanbhav.ruleEngine.model.QueryInput
import io.github.ayushmaanbhav.ruleEngine.model.QueryOutput
import io.github.ayushmaanbhav.ruleEngine.model.rule.Rule
import org.apache.logging.log4j.kotlin.Logging
import org.springframework.stereotype.Component

@Component
class CacheEnabledRuleEngine(private val cache: RuleEngineCache, private val evaluator: JsonLogicEvaluator) : Logging {
    fun evaluate(context: QueryContext, queries: List<Query>, input: QueryInput): QueryOutput {
        val rules = cache.get(
            context.identifier, QueryIdentifier(context.identifier, queries),
            { buildRuleDependencyGraph(context) }, { rdg -> rdg.computeExecutableRules(queries) }
        )
        return QueryOutput(evaluator.evaluate(rules, input.attributes))
    }

    private fun buildRuleDependencyGraph(context: QueryContext): DependencyGraph<Rule> {
        val ruleDependencyGraphBuilder = DependencyGraphBuilder<Rule>()
        context.rules.forEach(ruleDependencyGraphBuilder::visit)
        return ruleDependencyGraphBuilder.build()
    }
}
