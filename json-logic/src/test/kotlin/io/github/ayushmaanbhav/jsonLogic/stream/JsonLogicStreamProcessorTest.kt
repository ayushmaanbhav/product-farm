package io.github.ayushmaanbhav.jsonLogic.stream

import com.fasterxml.jackson.core.JsonFactory
import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.config.MathContext
import io.github.ayushmaanbhav.jsonLogic.config.StreamProcessingConfig
import io.kotest.core.spec.style.BehaviorSpec
import io.kotest.matchers.shouldBe
import java.math.BigDecimal

class JsonLogicStreamProcessorTest : BehaviorSpec({
    val evaluateEngine = JsonLogicEngine.Builder().build()
    val streamProcessor = JsonLogicStreamProcessor(StreamProcessingConfig(
        5, StreamProcessingConfig.DEFAULT_OPERATORS_INELIGIBLE_FOR_REDUCTION))

    given("Json parser from input stream of valid json logic along with data") {
        val inputStream = this.javaClass.getResourceAsStream("/test.json")

        `when`("on evaluation") {
            val result = streamProcessor
                .processTokens(JsonFactory().createParser(inputStream)) {
                    expression -> evaluateEngine.evaluate(expression, mapOf("a" to 2))
                }

            then("returns correct result") {
                result shouldBe BigDecimal(5).setScale(MathContext.DEFAULT.scale)
            }
        }
    }

    given("Json parser from input stream of invalid json") {
        val inputStream = "[]".byteInputStream()

        `when`("on evaluation") {
            var error: Exception? = null
            try {
                streamProcessor.processTokens(JsonFactory().createParser(inputStream)) { expression ->
                    evaluateEngine.evaluate(expression, null)
                }
            } catch (e: Exception) {
                error = e
            }

            then("exception should be invalid json logic") {
                error!!.javaClass shouldBe InvalidJsonLogicException::class.java
            }
        }
    }

    given("Json parser from input stream of invalid json logic case 2") {
        val inputStream = "{\"+\": {}}".byteInputStream()

        `when`("on evaluation") {
            var error: Exception? = null
            try {
                streamProcessor.processTokens(JsonFactory().createParser(inputStream)) { expression ->
                    evaluateEngine.evaluate(expression, null)
                }
            } catch (e: Exception) {
                error = e
            }

            then("exception should be invalid json logic") {
                error!!.javaClass shouldBe InvalidJsonLogicException::class.java
            }
        }
    }
})