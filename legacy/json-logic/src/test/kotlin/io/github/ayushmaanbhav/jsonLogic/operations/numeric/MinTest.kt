package io.github.ayushmaanbhav.jsonLogic.operations.numeric

import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.github.ayushmaanbhav.jsonLogic.stdlib.TestInput
import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimalDefaultContext
import io.github.ayushmaanbhav.jsonLogic.valueShouldBe
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData

class MinTest : FunSpec({
    val logicEngine = JsonLogicEngine.Builder().build()

    withData(
        nameFn = { input -> "Should evaluated ${input.expression} with given ${input.data} result in ${input.result}" },
        ts = listOf(
            TestInput(
                expression = mapOf("min" to listOf(1, 2, 3)),
                result = JsonLogicResult.Success(1.toBigDecimalDefaultContext())
            ),
            TestInput(
                expression = mapOf("min" to listOf("1", "0.2", 0.3)),
                result = JsonLogicResult.Success(0.2.toBigDecimalDefaultContext())
            ),
            TestInput(
                expression = mapOf("min" to listOf("-2", "0.2", 0.3)),
                result = JsonLogicResult.Success(-2.toBigDecimalDefaultContext())
            ),
            TestInput(
                expression = mapOf("min" to listOf(1, 3, 3)),
                result = JsonLogicResult.Success(1.toBigDecimalDefaultContext())
            ),
            TestInput(
                expression = mapOf("min" to listOf(3, 2, 1)),
                result = JsonLogicResult.Success(1.toBigDecimalDefaultContext())
            ),
            TestInput(
                expression = mapOf("min" to listOf(1)),
                result = JsonLogicResult.Success(1.toBigDecimalDefaultContext())
            ),
            TestInput(
                expression = mapOf("min" to listOf("1", 2)),
                result = JsonLogicResult.Success(1.toBigDecimalDefaultContext())
            ),
            TestInput(
                expression = mapOf("min" to listOf(Int.MIN_VALUE - 1L, Int.MIN_VALUE, 0)),
                result = JsonLogicResult.Success((Int.MIN_VALUE - 1L).toBigDecimalDefaultContext())
            ),
            TestInput(
                expression = mapOf("min" to listOf(Int.MAX_VALUE + 1L, Int.MAX_VALUE, 0)),
                result = JsonLogicResult.Success(0.toBigDecimalDefaultContext())
            ),
            TestInput(
                expression = mapOf("min" to listOf(1, "banana")),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("min" to listOf(1, "banana", listOf(1, 2))),
                result = JsonLogicResult.Failure.NullResult
            ),
        )
        // given
    ) { testInput: TestInput ->
        // when
        val evaluationResult = logicEngine.evaluate(testInput.expression, testInput.data)

        // then
        evaluationResult valueShouldBe testInput.result
    }
})
