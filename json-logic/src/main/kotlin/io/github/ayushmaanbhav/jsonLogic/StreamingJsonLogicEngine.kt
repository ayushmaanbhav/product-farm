package io.github.ayushmaanbhav.jsonLogic

import com.fasterxml.jackson.core.JsonFactory
import io.github.ayushmaanbhav.jsonLogic.stream.InvalidJsonLogicException
import io.github.ayushmaanbhav.jsonLogic.stream.JsonLogicStreamProcessor
import java.io.InputStream

internal class StreamingJsonLogicEngine(
    private val jsonLogicEngine: JsonLogicEngine, private val streamProcessor: JsonLogicStreamProcessor
) : JsonLogicEngine {
    override fun evaluate(expression: Map<String, Any?>, data: Any?): JsonLogicResult = jsonLogicEngine.evaluate(expression, data)

    override fun evaluate(inputStream: InputStream, data: Any?): JsonLogicResult {
        val jsonParser = JsonFactory().createParser(inputStream)
        return try {
            when (val result = jsonParser.use { streamProcessor.processTokens(it) { expression -> evaluate(expression, data) } }) {
                null -> JsonLogicResult.Failure.NullResult
                else -> JsonLogicResult.Success(result)
            }
        } catch (e: Exception) {
            when (e) {
                is InvalidJsonLogicException -> JsonLogicResult.Failure.InvalidFormat
                else -> JsonLogicResult.Failure.StreamIOError
            }
        }
    }
}
