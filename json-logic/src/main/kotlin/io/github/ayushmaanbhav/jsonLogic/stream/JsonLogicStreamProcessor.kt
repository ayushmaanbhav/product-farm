package io.github.ayushmaanbhav.jsonLogic.stream

import com.fasterxml.jackson.core.JsonParser
import com.fasterxml.jackson.core.JsonToken
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.github.ayushmaanbhav.jsonLogic.config.StreamProcessingConfig
import io.github.ayushmaanbhav.jsonLogic.utils.convertTokensToJsonStruct
import io.github.ayushmaanbhav.jsonLogic.utils.getCurrentValueFromJsonParser
import io.github.ayushmaanbhav.jsonLogic.utils.getJsonTokenFromValue
import io.github.ayushmaanbhav.jsonLogic.utils.getValueFromJsonLogicResult
import io.github.ayushmaanbhav.jsonLogic.utils.type.JsonObject
import io.github.ayushmaanbhav.jsonLogic.utils.type.LinkList
import io.github.ayushmaanbhav.jsonLogic.utils.type.MutablePair
import java.util.LinkedList

internal class JsonLogicStreamProcessor(private val streamProcessingConfig: StreamProcessingConfig) {
    fun processTokens(parser: JsonParser, evaluate: (Map<String, Any?>) -> JsonLogicResult): Any? {
        val tokenStack = LinkList<MutablePair<JsonToken, Any?>>()
        val objectStartTokenStack = LinkedList<LinkList.Node<MutablePair<JsonToken, Any?>>>()
        while (parser.nextToken() != null) {
            val currentValue = getCurrentValueFromJsonParser(parser)
            tokenStack.addLast(MutablePair(parser.currentToken, currentValue))
            if (parser.currentToken == JsonToken.START_OBJECT) {
                objectStartTokenStack.addLast(tokenStack.getLastNode())
                checkParentAndAddNonReducibleFlag(objectStartTokenStack)
            } else if (parser.currentToken == JsonToken.END_OBJECT) {
                mustNotBeEmpty(objectStartTokenStack)
                if (isParentObjectNonReducible(objectStartTokenStack) && hasNotExceededMaxLimit(objectStartTokenStack)) {
                    objectStartTokenStack.removeLast()
                    continue
                }
                val subList = tokenStack.subList(objectStartTokenStack.removeLast(), tokenStack.getLastNode())
                val expressionMap = (convertTokensToJsonStruct(subList) as JsonObject).value
                val reducedValue = getValueFromJsonLogicResult(evaluate(expressionMap))
                val reducedValueToken = getJsonTokenFromValue(reducedValue)
                subList.clear()
                tokenStack.addLast(MutablePair(reducedValueToken, reducedValue))
            } else if (parser.currentToken == JsonToken.FIELD_NAME && isNotEligibleForReduction(currentValue as String)) {
                mustNotBeEmpty(objectStartTokenStack)
                val depth = if (hasNonReducibleFlag(objectStartTokenStack.last.data)) getNonReducibleFlagDepth(objectStartTokenStack.last.data) else 0
                addNonReducibleFlag(objectStartTokenStack.last.data, depth + 1)
            }
        }
        if (tokenStack.hasSingleElement().not()) {
            throw InvalidJsonLogicException("Invalid json logic, result should have single element")
        }
        return tokenStack.getLastNode().data.second
    }

    private fun checkParentAndAddNonReducibleFlag(objectStartTokenStack: LinkedList<LinkList.Node<MutablePair<JsonToken, Any?>>>) {
        if (isParentObjectNonReducible(objectStartTokenStack)) {
            val parentDepth = getNonReducibleFlagDepth(objectStartTokenStack[objectStartTokenStack.size - 2].data)
            addNonReducibleFlag(objectStartTokenStack.last.data, parentDepth + 1)
        }
    }

    private fun isParentObjectNonReducible(objectStartTokenStack: LinkedList<LinkList.Node<MutablePair<JsonToken, Any?>>>): Boolean =
        objectStartTokenStack.size > 1 && hasNonReducibleFlag(objectStartTokenStack[objectStartTokenStack.size - 2].data)

    private fun hasNotExceededMaxLimit(objectStartTokenStack: LinkedList<LinkList.Node<MutablePair<JsonToken, Any?>>>): Boolean =
        getNonReducibleFlagDepth(objectStartTokenStack.last.data) <= streamProcessingConfig.maxOperatorStackLimitWithoutReduction

    private fun addNonReducibleFlag(tokenPair: MutablePair<JsonToken, Any?>, depth: Int) {
        tokenPair.second = mapOf(NON_REDUCIBLE to depth)
    }

    private fun hasNonReducibleFlag(tokenPair: MutablePair<JsonToken, Any?>): Boolean =
        tokenPair.second is Map<*, *> && (tokenPair.second as Map<*, *>).containsKey(NON_REDUCIBLE)

    private fun getNonReducibleFlagDepth(tokenPair: MutablePair<JsonToken, Any?>): Int =
        (tokenPair.second as Map<*, *>)[NON_REDUCIBLE] as Int

    private fun isNotEligibleForReduction(fieldName: String): Boolean {
        return streamProcessingConfig.operatorsIneligibleForReduction.contains(fieldName)
    }

    private fun mustNotBeEmpty(objectStartTokenStack: LinkedList<LinkList.Node<MutablePair<JsonToken, Any?>>>) {
        if (objectStartTokenStack.isEmpty()) {
            throw InvalidJsonLogicException("Invalid Json")
        }
    }

    companion object {
        const val NON_REDUCIBLE = "NON_REDUCIBLE"
    }
}
