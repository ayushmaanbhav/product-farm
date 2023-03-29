package io.github.ayushmaanbhav.jsonLogic.evaluation

import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.BehaviorSpec
import io.github.ayushmaanbhav.jsonLogic.api.JsonLogicException
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.config.MathContext

class CommonLogicEvaluatorTest : BehaviorSpec({
    val config = StandardLogicOperationConfig(MathContext.DEFAULT)
    val evaluator = CommonLogicEvaluator(config, LogicOperations())

    given("An unknown operation") {
        val logicExpression = mapOf("+" to listOf(2, mapOf("unknown" to "3")))

        then("throws an exception on evaluation") {
            shouldThrow<JsonLogicException> {
                evaluator.evaluateLogic(logicExpression, null)
            }
        }
    }
})
