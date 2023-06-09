package io.github.ayushmaanbhav.jsonLogic.operations.data

import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.github.ayushmaanbhav.jsonLogic.stdlib.TestInput
import io.github.ayushmaanbhav.jsonLogic.valueShouldBe
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData

class VarTest : FunSpec({
    val logicEngine = JsonLogicEngine.Builder().build()

    withData(
        nameFn = { input -> "Should evaluated ${input.expression} with given ${input.data} result in ${input.result}" },
        ts = listOf(
            TestInput(
                expression = mapOf("var" to "b"),
                data = mapOf("a" to 1),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(expression = mapOf("var" to "a"), result = JsonLogicResult.Failure.NullResult),
            TestInput(
                expression = mapOf("var" to listOf(listOf(""))),
                data = listOf(1, 2, 3, 4),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("var" to listOf(listOf(null))),
                data = listOf(1, 2, 3, 4),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("var" to listOf("b")),
                data = mapOf("a" to 1),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(expression = mapOf("var" to listOf("b")), result = JsonLogicResult.Failure.NullResult),
            TestInput(
                expression = mapOf("var" to listOf(emptyList<Any>())),
                data = listOf(1, 2, 3, 4),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(expression = mapOf("var" to "a.b.c"), result = JsonLogicResult.Failure.NullResult),
            TestInput(
                expression = mapOf("var" to "a.b.c"),
                data = mapOf("a" to null),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("var" to "a.q"),
                data = mapOf("a" to mapOf("b" to "c")),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("var" to "a.b.c"),
                data = mapOf("a" to mapOf("b" to null)),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf(
                    "var" to listOf(
                        mapOf(
                            "if" to listOf(
                                mapOf("<" to listOf(mapOf("var" to "temp"), 110)), "pie.filling", "pie.eta"
                            )
                        )
                    )
                ),
                data = mapOf("temp" to 100, "pie" to mapOf("filling" to "apple", "eta" to "60s")),
                result = JsonLogicResult.Success("apple")
            ),
            TestInput(
                expression = mapOf("var" to emptyList<Any>()),
                data = mapOf("a" to "apple", "b" to "banana"),
                result = JsonLogicResult.Success(mapOf("a" to "apple", "b" to "banana"))
            ),
            TestInput(
                expression = mapOf("var" to emptyList<Any>()),
                data = mapOf("a" to "apple", "b" to listOf("banana", "beer")),
                result = JsonLogicResult.Success(mapOf("a" to "apple", "b" to listOf("banana", "beer")))
            ),
            TestInput(
                expression = mapOf("var" to listOf("a")),
                data = mapOf("a" to 1),
                result = JsonLogicResult.Success(1)
            ),
            TestInput(
                expression = mapOf("var" to "a"),
                data = mapOf("a" to 1),
                result = JsonLogicResult.Success(1)
            ),
            TestInput(expression = mapOf("var" to listOf("a", 1)), result = JsonLogicResult.Success(1)),
            TestInput(expression = mapOf("var" to listOf("a", 1, 2)), result = JsonLogicResult.Success(1)),
            TestInput(
                expression = mapOf("var" to listOf("b", 2)),
                data = mapOf("a" to 1),
                result = JsonLogicResult.Success(2)
            ),
            TestInput(
                expression = mapOf("var" to "a.b"),
                data = mapOf("a" to mapOf("b" to "c")),
                result = JsonLogicResult.Success("c")
            ),
            TestInput(
                expression = mapOf("var" to listOf("a.q", 9)),
                data = mapOf("a" to mapOf("b" to "c")),
                result = JsonLogicResult.Success(9)
            ),
            TestInput(
                expression = mapOf("var" to listOf("a.b", 9)),
                data = mapOf("a" to mapOf("b" to "c")),
                result = JsonLogicResult.Success("c")
            ),
            TestInput(
                expression = mapOf("var" to 1),
                data = listOf("apple", "banana"),
                result = JsonLogicResult.Success("banana")
            ),
            TestInput(
                expression = mapOf("var" to "1"),
                data = listOf("apple", "banana"),
                result = JsonLogicResult.Success("banana")
            ),
            TestInput(
                expression = mapOf("var" to "1.1"),
                data = listOf("apple", listOf("banana", "beer")),
                result = JsonLogicResult.Success("beer")
            ),
            TestInput(expression = mapOf("var" to ""), data = 1, result = JsonLogicResult.Success(1)),
            TestInput(
                expression = mapOf("var" to ""),
                data = listOf(1, 2, 3, 4),
                result = JsonLogicResult.Success(listOf(1, 2, 3, 4))
            ),
            TestInput(
                expression = mapOf("var" to emptyList<Any>()),
                data = listOf(1, 2, 3, 4),
                result = JsonLogicResult.Success(listOf(1, 2, 3, 4))
            ),
            TestInput(
                expression = mapOf("var" to null),
                data = listOf(1, 2, 3, 4),
                result = JsonLogicResult.Success(listOf(1, 2, 3, 4))
            ),
            TestInput(
                expression = mapOf("var" to listOf(null)),
                data = listOf(1, 2, 3, 4),
                result = JsonLogicResult.Success(listOf(1, 2, 3, 4))
            ),
            TestInput(
                expression = mapOf("var" to listOf("")),
                data = listOf(1, 2, 3, 4),
                result = JsonLogicResult.Success(listOf(1, 2, 3, 4))
            ),
            TestInput(
                expression = mapOf("var" to listOf(1)),
                data = listOf(1, 2, 3, 4),
                result = JsonLogicResult.Success(2)
            ),
            TestInput(
                expression = mapOf("var" to listOf(listOf(1))),
                data = listOf(1, 2, 3, 4),
                result = JsonLogicResult.Success(2)
            ),
            TestInput(
                expression = mapOf("var" to listOf(listOf(1), 2)),
                data = listOf(1, 2, 3, 4),
                result = JsonLogicResult.Success(2)
            ),
            TestInput(
                expression = mapOf("var" to listOf(1, 2)),
                data = listOf(1, 2, 3, 4),
                result = JsonLogicResult.Success(2)
            ),
            TestInput(expression = mapOf("var" to null), data = 1, result = JsonLogicResult.Success(1)),
            TestInput(
                expression = mapOf("var" to null),
                data = mapOf("a" to "apple", "b" to "banana"),
                result = JsonLogicResult.Success(mapOf("a" to "apple", "b" to "banana"))
            ),
            TestInput(
                expression = mapOf("var" to null),
                data = mapOf("a" to "apple", "b" to listOf("banana", "beer")),
                result = JsonLogicResult.Success(mapOf("a" to "apple", "b" to listOf("banana", "beer")))
            ),
            TestInput(
                expression = mapOf("var" to null),
                data = listOf("apple", "banana"),
                result = JsonLogicResult.Success(listOf("apple", "banana"))
            ),
            TestInput(
                expression = mapOf("var" to null),
                data = listOf("apple", 1, null),
                result = JsonLogicResult.Success(listOf("apple", 1, null))
            ),
            TestInput(
                expression = mapOf("var" to null),
                data = listOf("apple", listOf("banana", "beer")),
                result = JsonLogicResult.Success(listOf("apple", listOf("banana", "beer")))
            ),
            TestInput(
                expression = mapOf("var" to emptyList<Any>()),
                data = 1,
                result = JsonLogicResult.Success(1)
            ),
            TestInput(
                expression = mapOf("var" to emptyList<Any>()),
                data = listOf("apple", "banana"),
                result = JsonLogicResult.Success(listOf("apple", "banana"))
            ),
            TestInput(
                expression = mapOf("var" to "1"),
                data = listOf("apple", listOf("banana", "beer")),
                result = JsonLogicResult.Success(listOf("banana", "beer"))
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
