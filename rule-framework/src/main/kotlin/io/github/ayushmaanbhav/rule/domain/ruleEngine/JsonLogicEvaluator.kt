package io.github.ayushmaanbhav.rule.domain.ruleEngine

import com.fasterxml.jackson.core.type.TypeReference
import com.fasterxml.jackson.databind.ObjectMapper
import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.github.ayushmaanbhav.rule.domain.ruleEngine.api.EvaluationEngine
import io.github.ayushmaanbhav.rule.domain.ruleEngine.exception.RuleEngineException
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.RuleEngineConfig
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.rule.Rule
import java.util.LinkedHashMap
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
            runCatching {
                val expression = objectMapper.readValue(rule.getExpression(), mapTypeReference)
                when (val jsonLogicResult = jsonLogic.evaluate(expression, context)) {
                    is JsonLogicResult.Failure.NullResult -> logger.debug("Ignoring rule gave empty output: ${rule.getId()}")
                    is JsonLogicResult.Failure -> throw RuleEngineException(
                        "Error occurred while running rule: ${rule.getId()}, ${jsonLogicResult.javaClass.name}"
                    )

                    is JsonLogicResult.Success -> {
                        val output = objectMapper.convertValue(jsonLogicResult.value, mapTypeReference)
                        output.forEach { (key: String, value: Any?) ->
                            if (context.containsKey(key)) {
                                throw RuleEngineException("Duplicate context key while running rule: ${rule.getId()}")
                            }
                            context[key] = value
                            allOutput[key] = value
                        }
                    }
                }
            }.onFailure { throw RuleEngineException("Error occurred while running rule: ${rule.getId()}", it) }
        }

        fun result(): LinkedHashMap<String, Any?> = allOutput
    }

    companion object {
        private val mapTypeReference = object : TypeReference<Map<String, Any?>>() {}
    }
}
