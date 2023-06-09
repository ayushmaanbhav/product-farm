package io.github.ayushmaanbhav.jsonLogic.operations.array

import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.github.ayushmaanbhav.jsonLogic.stdlib.TestInput
import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimalDefaultContext
import io.github.ayushmaanbhav.jsonLogic.valueShouldBe
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData

class ReduceTest : FunSpec({
    val logicEngine = JsonLogicEngine.Builder().build()

    withData(
        nameFn = { input -> "Should evaluated reduce expression with given ${input.data} result in ${input.result}" },
        ts = listOf(
            TestInput(
                expression = mapOf(
                    "reduce" to listOf(
                        listOf(1, 5, mapOf("var" to "A")),
                        mapOf("+" to listOf(mapOf("var" to "current"), mapOf("var" to "accumulator"))),
                        9
                    )
                ),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf(
                    "reduce" to listOf(
                        listOf(1, 5, mapOf("var" to "b")),
                        mapOf("+" to listOf(mapOf("var" to "current"), mapOf("var" to "accumulator"))),
                        9
                    )
                ),
                data = mapOf("b" to "banana"),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf(
                    "reduce" to listOf(mapOf("var" to "integers"))
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4)),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf(
                    "reduce" to listOf(
                        mapOf("*" to listOf(mapOf("var" to "current"), mapOf("var" to "accumulator"))),
                        0
                    )
                ),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("reduce" to listOf(0)),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("reduce" to emptyList<Any>()),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("reduce" to null),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf(
                    "reduce" to listOf(listOf(1, 2), null, 4)
                ),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("reduce" to listOf(mapOf("var" to "integers"), 0)),
                data = mapOf("integers" to listOf(1, 2, 3, 4)),
                result = JsonLogicResult.Success(0)
            ),
            TestInput(
                expression = mapOf(
                    "reduce" to listOf(1, 2, 3)
                ),
                result = JsonLogicResult.Success(3)
            ),
            TestInput(
                expression = mapOf(
                    "reduce" to listOf(listOf(1, 2), 3, 4)
                ),
                result = JsonLogicResult.Success(3)
            ),
            TestInput(
                expression = mapOf(
                    "reduce" to listOf(
                        mapOf("var" to "desserts"),
                        mapOf("+" to listOf(mapOf("var" to "accumulator"), mapOf("var" to "current.qty"))),
                        0
                    )
                ),
                data = mapOf(
                    "desserts" to listOf(
                        mapOf("name" to "apple", "qty" to 1),
                        mapOf("name" to "brownie", "qty" to 2),
                        mapOf("name" to "cupcake", "qty" to 3)
                    )
                ),
                result = JsonLogicResult.Success(6.toBigDecimalDefaultContext())
            ),
            TestInput(
                expression = mapOf(
                    "reduce" to listOf(
                        mapOf("var" to "integers"),
                        mapOf("*" to listOf(mapOf("var" to "current"), mapOf("var" to "accumulator"))),
                        0
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4)),
                result = JsonLogicResult.Success(0.toBigDecimalDefaultContext())
            ),
            TestInput(
                expression = mapOf(
                    "reduce" to listOf(
                        mapOf("var" to "integers"),
                        mapOf("*" to listOf(mapOf("var" to "current"), mapOf("var" to "accumulator"))),
                        1
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4)),
                result = JsonLogicResult.Success(24.toBigDecimalDefaultContext())
            ),
            TestInput(
                expression = mapOf(
                    "reduce" to listOf(
                        mapOf("var" to "integers"),
                        mapOf("+" to listOf(mapOf("var" to "current"), mapOf("var" to "accumulator"))),
                        0
                    )
                ),
                result = JsonLogicResult.Success(0)
            ),
            TestInput(
                expression = mapOf(
                    "reduce" to listOf(
                        mapOf("var" to "integers"),
                        mapOf("+" to listOf(mapOf("var" to "current"), mapOf("var" to "accumulator"))),
                        0
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4)),
                result = JsonLogicResult.Success(10.toBigDecimalDefaultContext())
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
