package io.github.ayushmaanbhav.jsonLogic.stdlib.array

import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult.Failure
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult.Success
import io.github.ayushmaanbhav.jsonLogic.stdlib.TestInput
import io.github.ayushmaanbhav.jsonLogic.utils.asBigDecimalList
import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimalDefaultContextList
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe

class SortTest : FunSpec({
    val operatorName = "sort"
    val logicEngine = JsonLogicEngine.Builder().addStandardOperation(operatorName, Sort).build()

    withData(
        nameFn = { input ->
            "Should evaluated $operatorName expression with given ${input.data} result in ${input.result}"
        },
        ts = listOf(
            TestInput(
                expression = mapOf(
                    operatorName to listOf(
                        mapOf(
                            "map" to listOf(
                                mapOf("var" to "integers", "var" to "integers"),
                                mapOf("%" to listOf(mapOf("var" to ""), 3)),
                                mapOf("var" to "integers", "var" to "integers"),
                            )
                        ), "desc"
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4, 5)),
                result = Success(listOf(2.0, 2.0, 1.0, 1.0, 0.0).toBigDecimalDefaultContextList())
            ),
            TestInput(
                expression = mapOf(
                    operatorName to listOf(
                        mapOf(
                            "map" to listOf(
                                mapOf("var" to "integers"),
                                mapOf("%" to listOf(mapOf("var" to ""), 2))
                            )
                        ), "asc"
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4, 5)),
                result = Success(listOf(0.0, 0.0, 1.0, 1.0, 1.0).toBigDecimalDefaultContextList())
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(1, 2, 3))),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(1.0, 2, 3.5), "desc")),
                result = Success(listOf(3.5, 2, 1.0).asBigDecimalList)
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(0, "desc")),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(0, 1, 2 , 3, "desc")),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(0.01, 0.01, 0.001), "desc")),
                result = Success(listOf(0.01, 0.01, 0.001).asBigDecimalList)
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf("0.1", "0.01", "0.001"), "desc")),
                result = Success(listOf("0.1", "0.01", "0.001"))
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf("0.1", "00.01", "000.001"), "asc")),
                result = Success(listOf("0.1", "00.01", "000.001"))
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(1, "true", 3), "asc")),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(1, 3, null), "asc")),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(1, 2), listOf(3, 4), "asc")),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(1, "2", 3), "asc")),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(1, 2, 3), "asc")),
                result = Success(listOf(1, 2, 3).asBigDecimalList)
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(1, 2, 3), "desc")),
                result = Success(listOf(3, 2, 1).asBigDecimalList)
            ),
            TestInput(
                expression = mapOf(operatorName to "banana"),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf("banana", "apple", "strawberry"), "asc")),
                result = Success(listOf("apple", "banana", "strawberry"))
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf("banana", 2, "strawberry"), "asc")),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf("banana", null, "strawberry"), "asc")),
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
            TestInput(
                expression = mapOf(operatorName to listOf(null, mapOf("var" to ""))),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(emptyList<String>(), mapOf("var" to ""))),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(false, false), mapOf("var" to ""))),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(true, false), "asc")),
                result = Success(listOf(false, true))
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(true), "asc")),
                result = Success(listOf(true))
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(false), "asc")),
                result = Success(listOf(false))
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(true, null), "asc")),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(true, "1"), "asc")),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(true, "true"), "asc")),
                result = Failure.NullResult
            ),
            TestInput(
                expression = mapOf(operatorName to listOf(listOf(true, false), "desc")),
                result = Success(listOf(true, false))
            ),
            TestInput(
                expression = mapOf(
                    operatorName to listOf(
                        mapOf("var" to "fruits"),
                        mapOf("==" to listOf(mapOf("var" to ""), "pineapple"))
                    )
                ),
                data = mapOf("fruits" to listOf("apple", "banana", "pineapple")),
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
