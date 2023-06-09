package io.github.ayushmaanbhav.jsonLogic.operations.numeric.unwrap

import io.github.ayushmaanbhav.jsonLogic.config.MathContext
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimalDefaultContext
import io.kotest.core.spec.style.BehaviorSpec
import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.shouldBe

class StrictUnwrapStrategyTest : BehaviorSpec({
    val config = StandardLogicOperationConfig(MathContext.DEFAULT)
    val strategyImplementation: StrictUnwrapStrategy = object : StrictUnwrapStrategy {}

    given("A false value") {
        val wrappedValue = listOf(false)
        `when`("unwrapped") {
            val unwrapResult = strategyImplementation.unwrapValue(config, wrappedValue)
            then("should be null") {
                unwrapResult shouldHaveSize 1
                unwrapResult.first() shouldBe null
            }
        }
    }

    given("A true value") {
        val wrappedValue = listOf(true)
        `when`("unwrapped") {
            val unwrapResult = strategyImplementation.unwrapValue(config, wrappedValue)
            then("should be null") {
                unwrapResult shouldHaveSize 1
                unwrapResult.first() shouldBe null
            }
        }
    }

    given("A null") {
        val wrappedValue = listOf(null)
        `when`("unwrapped") {
            val unwrapResult = strategyImplementation.unwrapValue(config, wrappedValue)
            then("should be null") {
                unwrapResult shouldHaveSize 1
                unwrapResult.first() shouldBe null
            }
        }
    }

    given("Any other value") {
        val wrappedValue = listOf(Pair("a", "apple"))
        `when`("unwrapped") {
            val unwrapResult = strategyImplementation.unwrapValue(config, wrappedValue)
            then("should return null") {
                unwrapResult shouldHaveSize 1
                unwrapResult.first() shouldBe null
            }
        }
    }

    given("A number string") {
        val wrappedValue = listOf("32.5")
        `when`("unwrapped") {
            val unwrapResult = strategyImplementation.unwrapValue(config, wrappedValue)
            then("should be equal to its number value") {
                unwrapResult shouldHaveSize 1
                unwrapResult.first() shouldBe 32.5.toBigDecimalDefaultContext()
            }
        }
    }

    given("A not number string") {
        val wrappedValue = listOf("Not a number")
        `when`("unwrapped") {
            val unwrapResult = strategyImplementation.unwrapValue(config, wrappedValue)
            then("should be null") {
                unwrapResult shouldHaveSize 1
                unwrapResult.first() shouldBe null
            }
        }
    }

    given("An empty list") {
        val wrappedValue = listOf(emptyList<String>())
        `when`("unwrapped") {
            val unwrapResult = strategyImplementation.unwrapValue(config, wrappedValue)
            then("should be null") {
                unwrapResult shouldHaveSize 1
                unwrapResult.first() shouldBe null
            }
        }
    }

    given("A more than 2 element list") {
        val wrappedValue = listOf(listOf(1, 2, "Not a number"))
        `when`("unwrapped") {
            val unwrapResult = strategyImplementation.unwrapValue(config, wrappedValue)
            then("should be equal to its first element") {
                unwrapResult shouldHaveSize 1
                unwrapResult.first() shouldBe 1.toBigDecimalDefaultContext()
            }
        }
    }

    given("A single element list") {
        val wrappedValue = listOf(listOf("20"))
        `when`("unwrapped") {
            val unwrapResult = strategyImplementation.unwrapValue(config, wrappedValue)
            then("should be equal to its element") {
                unwrapResult shouldHaveSize 1
                unwrapResult.first() shouldBe 20.toBigDecimalDefaultContext()
            }
        }
    }

    given("A nested single element list") {
        val wrappedValue = listOf(listOf(listOf("20")))
        `when`("unwrapped") {
            val unwrapResult = strategyImplementation.unwrapValue(config, wrappedValue)
            then("should be equal to its deepest element") {
                unwrapResult shouldHaveSize 1
                unwrapResult.first() shouldBe 20.toBigDecimalDefaultContext()
            }
        }
    }
})
