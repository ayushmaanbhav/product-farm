package io.github.ayushmaanbhav.jsonLogic.stdlib.string

import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult.Failure
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult.Success
import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.stdlib.TestInput
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe

class LowercaseTest : FunSpec({
    val operatorName = "lowercase"
    val logicEngine = JsonLogicEngine.Builder().addStandardOperation(operatorName, Lowercase).build()

    withData(
        nameFn = { input -> "Should evaluated ${input.expression} with given ${input.data} result in ${input.result}" },
        ts = listOf(
            TestInput(
                expression = mapOf(operatorName to "banana"),
                result = Success("banana")
            ),
            TestInput(
                expression = mapOf(operatorName to ""),
                result = Success("")
            ),
            TestInput(
                expression = mapOf(operatorName to " "),
                result = Success(" ")
            ),
            TestInput(
                expression = mapOf(operatorName to "123"),
                result = Success("123")
            ),
            TestInput(
                expression = mapOf(operatorName to "Test"),
                result = Success("test")
            ),
            TestInput(
                expression = mapOf(operatorName to "TEST ME!"),
                result = Success("test me!")
            ),
            TestInput(
                expression = mapOf(operatorName to mapOf("var" to "key")),
                data = mapOf("key" to "APPLE"),
                result = Success("apple")
            ),
            TestInput(
                expression = mapOf(operatorName to mapOf("var" to "key")),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to 1.3),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to null),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to true),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to emptyList<Any>()),
                result = Failure.NullResult
            ),
        )
        // given
    ) { testInput: TestInput ->
        // when
        val evaluationResult = logicEngine.evaluate(testInput.expression, testInput.data)

        // then
        evaluationResult shouldBe testInput.result
    }
})
