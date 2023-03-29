package io.github.ayushmaanbhav.rule.domain.ruleEngine

import com.fasterxml.jackson.core.type.TypeReference
import com.fasterxml.jackson.databind.ObjectMapper
import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.github.ayushmaanbhav.rule.domain.ruleEngine.api.EvaluationEngine
import io.github.ayushmaanbhav.rule.domain.ruleEngine.exception.RuleEngineException
import io.github.ayushmaanbhav.rule.domain.ruleEngine.config.RuleEngineConfig
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.rule.Rule
import org.apache.logging.log4j.kotlin.Logging

class JsonLogicEvaluator(config: RuleEngineConfig) : EvaluationEngine, Logging {
    private val objectMapper: ObjectMapper = config.objectMapper
    private val jsonLogic = JsonLogicEngine.Builder()
        .addLogger { any -> logger.debug("json logic log : $any") }
        .addMathContext(config.mathContext).build()

    override fun evaluate(rules: List<Rule>, attributes: LinkedHashMap<String, Any?>): LinkedHashMap<String, Any?> {
        val visitor = Visitor(attributes)
        rules.forEach(visitor::visit)
        return visitor.result()
    }

    private inner class Visitor(attributes: LinkedHashMap<String, Any?>) {
        private val context: LinkedHashMap<String, Any?> = LinkedHashMap(attributes)
        private val allOutput: LinkedHashMap<String, Any?> = LinkedHashMap()

        fun visit(rule: Rule) {
            val expression = readExpression(rule.getId(), rule.getExpression())
            val result = runCatching { jsonLogic.evaluate(expression, context) }
                .getOrElse { throw RuleEngineException("Error occurred while running rule: ${rule.getId()}", it) }
            when (result) {
                is JsonLogicResult.Failure.NullResult -> logger.debug("Ignoring rule gave empty output: ${rule.getId()}")
                is JsonLogicResult.Failure -> throw RuleEngineException("Got failure on running rule: ${rule.getId()}, ${result.javaClass.name}")

                is JsonLogicResult.Success -> {
                    val output = readOutput(rule.getId(), result.value)
                    output.forEach { (key: String, value: Any?) ->
                        if (context.containsKey(key)) {
                            throw RuleEngineException("Duplicate context key found in rule output: ${rule.getId()}")
                        }
                        context[key] = value
                        allOutput[key] = value
                    }
                }
            }
        }

        private fun readExpression(ruleId: String, expression: String): LinkedHashMap<String, Any?> =
            runCatching { objectMapper.readValue(expression, mapTypeReference) }
                .getOrElse { throw RuleEngineException("Error occurred while reading rule expression: $ruleId", it) }

        private fun readOutput(ruleId: String, output: Any): LinkedHashMap<String, Any?> =
            runCatching { objectMapper.convertValue(output, mapTypeReference) }
                .getOrElse { throw RuleEngineException("Error occurred while reading rule engine output: $ruleId", it) }

        fun result(): LinkedHashMap<String, Any?> = allOutput
    }

    companion object {
        private val mapTypeReference = object : TypeReference<LinkedHashMap<String, Any?>>() {}
    }
}
