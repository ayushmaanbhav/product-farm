package io.github.ayushmaanbhav.jsonLogic.operations.string

import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.github.ayushmaanbhav.jsonLogic.stdlib.TestInput
import io.github.ayushmaanbhav.jsonLogic.valueShouldBe
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData

class CatTest : FunSpec({
    val logicEngine = JsonLogicEngine.Builder().build()

    withData(
        nameFn = { input -> "Should evaluated ${input.expression} with given ${input.data} result in ${input.result}" },
        ts = listOf(
            TestInput(
                expression = mapOf("cat" to "ice"),
                result = JsonLogicResult.Success("ice")
            ),
            TestInput(
                expression = mapOf("cat" to emptyList<String>()),
                result = JsonLogicResult.Success("")
            ),
            TestInput(
                expression = mapOf("cat" to listOf("ice")),
                result = JsonLogicResult.Success("ice")
            ),
            TestInput(
                expression = mapOf("cat" to listOf("ice", "cream")),
                result = JsonLogicResult.Success("icecream")
            ),
            TestInput(
                expression = mapOf("cat" to listOf(1, 2)),
                result = JsonLogicResult.Success("12")
            ),
            TestInput(
                expression = mapOf("cat" to listOf("Robocop", 2)),
                result = JsonLogicResult.Success("Robocop2")
            ),
            TestInput(
                expression = mapOf("cat" to listOf("Robocop", 2.0)),
                result = JsonLogicResult.Success("Robocop2")
            ),
            TestInput(
                expression = mapOf("cat" to listOf(true, "Robocop")),
                result = JsonLogicResult.Success("trueRobocop")
            ),
            TestInput(
                expression = mapOf("cat" to listOf("false", "Robocop")),
                result = JsonLogicResult.Success("falseRobocop")
            ),
            TestInput(
                expression = mapOf("cat" to listOf("we all scream for ", "ice", "cream")),
                result = JsonLogicResult.Success("we all scream for icecream")
            ),
            TestInput(
                expression = mapOf("cat" to listOf("we all scream for ", listOf("ice", "cream"))),
                result = JsonLogicResult.Success("we all scream for ice,cream")
            ),
            TestInput(
                expression = mapOf("cat" to listOf("easy as ", listOf(1, 2.0, "3"))),
                result = JsonLogicResult.Success("easy as 1,2,3")
            ),
            TestInput(
                expression = mapOf("cat" to listOf(listOf(2.0))),
                result = JsonLogicResult.Success("2")
            ),
            TestInput(
                expression = mapOf("cat" to listOf(2.5, 1)),
                result = JsonLogicResult.Success("2.51")
            ),
            TestInput(
                expression = mapOf("cat" to listOf("2.0")),
                result = JsonLogicResult.Success("2.0")
            ),
            TestInput(
                expression = mapOf("cat" to listOf("easy as ", listOf(1, 2, "3"))),
                result = JsonLogicResult.Success("easy as 1,2,3")
            ),
            TestInput(
                expression = mapOf("cat" to listOf("easy as ", listOf(null, listOf(true), "3"))),
                result = JsonLogicResult.Success("easy as ,true,3")
            ),
            TestInput(
                expression = mapOf("cat" to listOf(emptyList(), listOf(emptyList(), listOf(emptyList<String>())))),
                result = JsonLogicResult.Success("")
            ),
            TestInput(
                expression = mapOf("cat" to listOf("I love ", mapOf("var" to "filling"), " pie")),
                data = mapOf("filling" to "apple", "temp" to 110),
                result = JsonLogicResult.Success("I love apple pie")
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
