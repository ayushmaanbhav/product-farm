package io.github.ayushmaanbhav.jsonLogic.stdlib.array

import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.github.ayushmaanbhav.jsonLogic.stdlib.TestInput
import io.kotest.core.spec.style.FunSpec
import io.kotest.datatest.withData
import io.kotest.matchers.shouldBe

class FindTest : FunSpec({
    val logicEngine = JsonLogicEngine.Builder().addFunctionalOperation("find", Find).build()

    withData(
        nameFn = { input -> "Should evaluated ${input.expression} with given ${input.data} result in ${input.result}" },
        ts = listOf(
            TestInput(
                expression = mapOf("find" to 1.3),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("find" to "banana"),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("find" to null),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("find" to true),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("find" to emptyList<Any>()),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("find" to listOf(null, mapOf("var" to ""))),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("find" to listOf(emptyList<String>(), mapOf("var" to ""))),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("find" to listOf(listOf(false, false), mapOf("var" to ""))),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf("find" to listOf(listOf(true, false), mapOf("var" to ""))),
                result = JsonLogicResult.Success(true)
            ),
            TestInput(
                expression = mapOf("find" to listOf(listOf(true, false))),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf(
                    "find" to listOf(
                        mapOf(
                            "filter" to listOf(
                                mapOf("var" to "integers"),
                                mapOf("%" to listOf(mapOf("var" to ""), 2))
                            )
                        ), mapOf(">" to listOf(mapOf("var" to ""), 1))
                    )
                ),
                data = mapOf("integers" to listOf(1, 2, 3, 4, 5)),
                result = JsonLogicResult.Success(3)
            ),
            TestInput(
                expression = mapOf("find" to listOf(listOf(-1, 1, 2, 3), mapOf(">" to listOf(mapOf("var" to ""), 0)))),
                result = JsonLogicResult.Success(1)
            ),
            TestInput(
                expression = mapOf("find" to listOf(listOf(0, 0, 0, 0), mapOf("==" to listOf(mapOf("var" to ""), 0)))),
                result = JsonLogicResult.Success(0)
            ),
            TestInput(
                expression = mapOf("find" to listOf(listOf(0, 0, 0, 0), mapOf("===" to listOf(mapOf("var" to ""), 0)))),
                result = JsonLogicResult.Success(0)
            ),
            TestInput(
                expression = mapOf("find" to listOf(listOf(0, 0, 0, 0), mapOf("!=" to listOf(mapOf("var" to ""), 0)))),
                result = JsonLogicResult.Failure.NullResult
            ),
            TestInput(
                expression = mapOf(
                    "find" to listOf(
                        listOf(-1, "b", "a", 3),
                        mapOf(">" to listOf(mapOf("var" to ""), 0))
                    )
                ),
                result = JsonLogicResult.Success(3)
            ),
            TestInput(
                expression = mapOf("find" to listOf(listOf(-1, 1, 2, 3), mapOf("<" to listOf(mapOf("var" to ""), 0)))),
                result = JsonLogicResult.Success(-1)
            ),
            TestInput(
                expression = mapOf(
                    "find" to listOf(
                        listOf("banana", "apple"),
                        mapOf("==" to listOf(mapOf("var" to ""), "apple"))
                    )
                ),
                result = JsonLogicResult.Success("apple")
            ),
            TestInput(
                expression = mapOf(
                    "find" to listOf(
                        mapOf("var" to "fruits"),
                        mapOf("==" to listOf(mapOf("var" to ""), "pineapple"))
                    )
                ),
                data = mapOf("fruits" to listOf("apple", "banana", "pineapple")),
                result = JsonLogicResult.Success("pineapple")
            ),
            TestInput(
                expression = mapOf(
                    "find" to listOf(
                        listOf("a", "b", "c"),
                        mapOf("<" to listOf(mapOf("var" to ""), 0))
                    )
                ),
                result = JsonLogicResult.Failure.NullResult
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
