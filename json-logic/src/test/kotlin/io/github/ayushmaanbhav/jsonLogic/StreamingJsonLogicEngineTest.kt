package io.github.ayushmaanbhav.jsonLogic

import io.github.ayushmaanbhav.jsonLogic.stream.InvalidJsonLogicException
import io.github.ayushmaanbhav.jsonLogic.stream.JsonLogicStreamProcessor
import io.kotest.core.spec.style.BehaviorSpec
import io.kotest.matchers.shouldBe
import io.mockk.every
import io.mockk.mockk
import java.io.IOException
import java.io.InputStream

class StreamingJsonLogicEngineTest : BehaviorSpec({
    given("Json logic expression") {
        val successResult: JsonLogicResult = JsonLogicResult.Success(true)
        val streamProcessor: JsonLogicStreamProcessor = mockk()
        val engine = StreamingJsonLogicEngine(MockJsonLogicEngine(successResult), streamProcessor)
        val expression: Map<String, Any?> = mapOf()

        `when`("on evaluation") {
            val result = engine.evaluate(expression, null)

            then("returns expected result") {
                result shouldBe successResult
            }
        }
    }

    given("Json logic input stream and stream processor returns success") {
        val successResult: JsonLogicResult = JsonLogicResult.Success(true)
        val streamProcessor: JsonLogicStreamProcessor = mockk()
        val engine = StreamingJsonLogicEngine(MockJsonLogicEngine(successResult), streamProcessor)
        val inputStream: InputStream = "{}".byteInputStream()

        every { streamProcessor.processTokens(any(), any()) } returns successResult

        `when`("on evaluation") {
            val result = engine.evaluate(inputStream, null)

            then("returns expected result") {
                result shouldBe JsonLogicResult.Success(successResult)
            }
        }
    }

    given("Json logic input stream and stream processor returns null") {
        val successResult: JsonLogicResult = JsonLogicResult.Success(true)
        val streamProcessor: JsonLogicStreamProcessor = mockk()
        val engine = StreamingJsonLogicEngine(MockJsonLogicEngine(successResult), streamProcessor)
        val inputStream: InputStream = "{}".byteInputStream()

        every { streamProcessor.processTokens(any(), any()) } returns null

        `when`("on evaluation") {
            val result = engine.evaluate(inputStream, null)

            then("returns null result") {
                result shouldBe JsonLogicResult.Failure.NullResult
            }
        }
    }

    given("Json logic input stream and stream processor throws invalid json exception") {
        val successResult: JsonLogicResult = JsonLogicResult.Success(true)
        val streamProcessor: JsonLogicStreamProcessor = mockk()
        val engine = StreamingJsonLogicEngine(MockJsonLogicEngine(successResult), streamProcessor)
        val inputStream: InputStream = "{}".byteInputStream()

        every { streamProcessor.processTokens(any(), any()) } throws InvalidJsonLogicException("")

        `when`("on evaluation") {
            val result = engine.evaluate(inputStream, null)

            then("returns null result") {
                result shouldBe JsonLogicResult.Failure.InvalidFormat
            }
        }
    }

    given("Json logic input stream and stream processor throws io exception") {
        val successResult: JsonLogicResult = JsonLogicResult.Success(true)
        val streamProcessor: JsonLogicStreamProcessor = mockk()
        val engine = StreamingJsonLogicEngine(MockJsonLogicEngine(successResult), streamProcessor)
        val inputStream: InputStream = "{}".byteInputStream()

        every { streamProcessor.processTokens(any(), any()) } throws IOException("")

        `when`("on evaluation") {
            val result = engine.evaluate(inputStream, null)

            then("returns null result") {
                result shouldBe JsonLogicResult.Failure.StreamIOError
            }
        }
    }
})

internal class MockJsonLogicEngine(private val result: JsonLogicResult) : JsonLogicEngine {
    override fun evaluate(expression: Map<String, Any?>, data: Any?): JsonLogicResult = result
}
