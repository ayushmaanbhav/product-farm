package io.github.ayushmaanbhav.jsonLogic.operations.logic.equals.strict

import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.github.ayushmaanbhav.jsonLogic.stdlib.TestInput
import io.github.ayushmaanbhav.jsonLogic.valueShouldBe
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData

class NotStrictEqualsTest : FunSpec({
    val logicEngine = JsonLogicEngine.Builder().build()

    withData(
        nameFn = { input -> "Should evaluated ${input.expression} with given ${input.data} result in ${input.result}" },
        ts = listOf(
            TestInput(expression = mapOf("!==" to listOf(1, 1)), result = JsonLogicResult.Success(false)),
            TestInput(expression = mapOf("!==" to listOf(1, 1.0000)), result = JsonLogicResult.Success(false)),
            TestInput(expression = mapOf("!==" to listOf(0, 0.0000)), result = JsonLogicResult.Success(false)),
            TestInput(expression = mapOf("!==" to listOf(1, "1.0000")), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf("1.0000", "1.0000")), result = JsonLogicResult.Success(false)),
            TestInput(expression = mapOf("!==" to listOf(-1, -1.0000)), result = JsonLogicResult.Success(false)),
            TestInput(expression = mapOf("!==" to listOf(-1, -1)), result = JsonLogicResult.Success(false)),
            TestInput(expression = mapOf("!==" to listOf("-1", "-1")), result = JsonLogicResult.Success(false)),
            TestInput(expression = mapOf("!==" to listOf(-1, "-1")), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf(-2, "-2")), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf(1, "1")), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf(1, 2)), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf(null, 2)), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf(null, true)), result = JsonLogicResult.Success(true)),
            TestInput(
                expression = mapOf("!==" to listOf(null, false)),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(null, "false")),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(null, "true")),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(false, "false")),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(false, listOf("false"))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(false, listOf(false))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(emptyList<Any>(), listOf(emptyList<Any>()))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(emptyList<Any>(), null)),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(emptyList(), emptyList<Any>())),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf("null", null)),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(true, "true")),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(expression = mapOf("!==" to listOf(1, null)), result = JsonLogicResult.Success(true)),
            TestInput(
                expression = mapOf("!==" to listOf(listOf("banana"), null)),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(listOf("banana"), true)),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(listOf("banana"), false)),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(true, false)),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(false, true)),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(true, true)),
                result = JsonLogicResult.Success(false)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(1, listOf("1"))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(1, listOf(1))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(expression = mapOf("!==" to listOf(1, false)), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf(-1, false)), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf(0, false)), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf(0, true)), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf(1, true)), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf(-1, true)), result = JsonLogicResult.Success(true)),
            TestInput(
                expression = mapOf("!==" to listOf(true, true, "false")),
                result = JsonLogicResult.Success(false)
            ),
            TestInput(expression = mapOf("!==" to listOf(1, 0, 1)), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf(1)), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf(true)), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to true), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to false), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf("true")), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf("banana")), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to "banana"), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to listOf(null)), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to null), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to ""), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to "     "), result = JsonLogicResult.Success(true)),
            TestInput(expression = mapOf("!==" to emptyList<Any>()), result = JsonLogicResult.Success(false)),
            TestInput(
                expression = mapOf("!==" to listOf(emptyList<Any>())),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(null, null)),
                result = JsonLogicResult.Success(false)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(1, listOf(listOf("1")))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf("banana", listOf(listOf("banana")))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf("banana", listOf("banana", "banana"))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(listOf(null), listOf(null))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(null, listOf(null))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(1, listOf(null))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(0, listOf(null))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(true, listOf(null))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(false, listOf(null))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(-1, listOf(null))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(0.5, listOf(null))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(expression = mapOf("!==" to listOf("", "")), result = JsonLogicResult.Success(false)),
            TestInput(expression = mapOf("!==" to listOf("", "    ")), result = JsonLogicResult.Success(true)),
            TestInput(
                expression = mapOf("!==" to listOf("", emptyList<String>())),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf("", listOf(""))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(listOf(""), listOf(""))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf("", listOf(listOf("")))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(false, emptyList<String>())),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(0, emptyList<String>())),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf("0", emptyList<String>())),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf("0.0", emptyList<String>())),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(1, emptyList<String>())),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf("1", emptyList<String>())),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf("1.0", emptyList<String>())),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf(-1, emptyList<String>())),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf("-1", emptyList<String>())),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("!==" to listOf("-1.0", emptyList<String>())),
                result = JsonLogicResult.Success(true)
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
