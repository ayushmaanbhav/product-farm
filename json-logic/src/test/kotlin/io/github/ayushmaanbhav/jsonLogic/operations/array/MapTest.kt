package io.github.ayushmaanbhav.jsonLogic.operations.array

import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.github.ayushmaanbhav.jsonLogic.stdlib.TestInput
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimalDefaultContext
import io.github.ayushmaanbhav.jsonLogic.valueShouldBe

class MapTest : FunSpec({
    val logicEngine = JsonLogicEngine.Builder().build()

    withData(
        nameFn = { input -> "Should evaluated ${input.expression} with given ${input.data} result in ${input.result}" },
        ts = listOf(
            TestInput(
                expression = mapOf(
                    "map" to listOf(
                        listOf(1, 5, mapOf("var" to "A")),
                        mapOf("+" to listOf(mapOf("var" to ""), 2))
                    )
                ),
                result = JsonLogicResult.Success(listOf(3.toBigDecimalDefaultContext(), 7.toBigDecimalDefaultContext(), null))
            ),
            TestInput(
                expression = mapOf(
                    "map" to listOf(
                        mapOf("var" to "desserts"),
                        mapOf("var" to "qty")
                    )
                ),
                data = mapOf(
                    "desserts" to listOf(
                        mapOf("name" to "apple", "qty" to 1),
                        mapOf("name" to "brownie", "qty" to 2),
                        mapOf("name" to "cupcake", "qty" to 3)
                    )
                ),
                result = JsonLogicResult.Success(listOf(1, 2, 3))
            ),
            TestInput(
                expression = mapOf(
                    "map" to listOf(
                        mapOf("var" to "integers"),
                        mapOf("*" to listOf(mapOf("var" to ""), 2))
                    )
                ),
                result = JsonLogicResult.Success(emptyList<Any>())
            ),
            TestInput(
                expression = mapOf(
                    "map" to listOf(
                        mapOf("var" to "integers"),
                        mapOf("*" to listOf(mapOf("var" to ""), 2))
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4, 5)),
                result = JsonLogicResult.Success(listOf(
                    2.toBigDecimalDefaultContext(), 4.toBigDecimalDefaultContext(), 6.toBigDecimalDefaultContext(),
                    8.toBigDecimalDefaultContext(), 10.toBigDecimalDefaultContext()))
            ),
            TestInput(
                expression = mapOf(
                    "map" to listOf(
                        1,
                        mapOf("var" to "integers"),
                        mapOf("*" to listOf(mapOf("var" to ""), 2))
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4, 5)),
                result = JsonLogicResult.Success(emptyList<Any>())
            ),
            TestInput(
                expression = mapOf(
                    "map" to listOf(
                        listOf(null),
                        1,
                        mapOf("var" to "integers"),
                        mapOf("*" to listOf(mapOf("var" to ""), 2))
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4, 5)),
                result = JsonLogicResult.Success(listOf(1))
            ),
            TestInput(
                expression = mapOf(
                    "map" to listOf(
                        listOf(2, "banana"),
                        mapOf("*" to listOf(mapOf("var" to ""), 2))
                    )
                ),
                result = JsonLogicResult.Success(listOf(4.toBigDecimalDefaultContext(), null))
            ),
            TestInput(
                expression = mapOf(
                    "map" to listOf(
                        2,
                        mapOf("var" to "integers"),
                        mapOf("*" to listOf(mapOf("var" to ""), 2))
                    )
                ),
                result = JsonLogicResult.Success(emptyList<Any>())
            ),
            TestInput(
                expression = mapOf("map" to listOf(1, 2, 3, 4, 5)),
                result = JsonLogicResult.Success(emptyList<Any>())
            ),
            TestInput(
                expression = mapOf("map" to listOf(listOf(1, 2, 3, 4, 5))),
                result = JsonLogicResult.Success(listOf(null, null, null, null, null))
            ),
            TestInput(
                expression = mapOf("map" to listOf(listOf(1, 2, 3), listOf(1, 2))),
                result = JsonLogicResult.Success(listOf(listOf(1, 2), listOf(1, 2), listOf(1, 2)))
            ),
            TestInput(
                expression = mapOf(
                    "map" to listOf(
                        listOf(mapOf("var" to "integers"), 1),
                        mapOf("*" to listOf(mapOf("var" to ""), 2))
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4, 5)),
                result = JsonLogicResult.Success(listOf(2.toBigDecimalDefaultContext(), 2.toBigDecimalDefaultContext()))
            ),
            TestInput(
                expression = mapOf(
                    "map" to listOf(
                        mapOf("var" to "integers"),
                        mapOf("%" to listOf(mapOf("var" to ""), 2))
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4, 5)),
                result = JsonLogicResult.Success(listOf(
                    1.toBigDecimalDefaultContext(), 0.toBigDecimalDefaultContext(), 1.toBigDecimalDefaultContext(), 0.toBigDecimalDefaultContext(),
                    1.toBigDecimalDefaultContext()))
            ),
            TestInput(
                expression = mapOf(
                    "map" to listOf(
                        mapOf("var" to "integers", "var" to "integers"),
                        mapOf("%" to listOf(mapOf("var" to ""), 2))
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4, 5)),
                result = JsonLogicResult.Success(listOf(1.toBigDecimalDefaultContext(), 0.toBigDecimalDefaultContext(), 1.toBigDecimalDefaultContext(), 0.toBigDecimalDefaultContext(), 1.toBigDecimalDefaultContext()))
            ),
            TestInput(
                expression = mapOf(
                    "map" to listOf(
                        mapOf("%" to listOf(mapOf("var" to ""), 2))
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4, 5)),
                result = JsonLogicResult.Success(emptyList<Any>())
            ),
            TestInput(
                expression = mapOf("map" to emptyList<Any>()),
                result = JsonLogicResult.Success(emptyList<Any>())
            ),
            TestInput(
                expression = mapOf("map" to null),
                result = JsonLogicResult.Success(emptyList<Any>())
            ),
            TestInput(
                expression = mapOf("map" to "banana"),
                result = JsonLogicResult.Success(emptyList<Any>())
            ),
            TestInput(
                expression = mapOf(
                    "map" to listOf(
                        mapOf("var" to "integers", "var" to "integers"),
                        mapOf("%" to listOf(mapOf("var" to ""), 2)),
                        mapOf("var" to "integers", "var" to "integers"),
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4, 5)),
                result = JsonLogicResult.Success(listOf(1.toBigDecimalDefaultContext(), 0.toBigDecimalDefaultContext(), 1.toBigDecimalDefaultContext(), 0.toBigDecimalDefaultContext(), 1.toBigDecimalDefaultContext()))
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
