package io.github.ayushmaanbhav.jsonLogic.utils

import com.fasterxml.jackson.core.JsonParser
import com.fasterxml.jackson.core.JsonToken
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.github.ayushmaanbhav.jsonLogic.stream.InvalidJsonLogicException
import io.github.ayushmaanbhav.jsonLogic.utils.type.JsonArray
import io.github.ayushmaanbhav.jsonLogic.utils.type.JsonField
import io.github.ayushmaanbhav.jsonLogic.utils.type.JsonObject
import io.github.ayushmaanbhav.jsonLogic.utils.type.JsonStruct
import io.github.ayushmaanbhav.jsonLogic.utils.type.JsonValue
import io.github.ayushmaanbhav.jsonLogic.utils.type.LinkList
import io.github.ayushmaanbhav.jsonLogic.utils.type.MutablePair
import java.util.LinkedList

internal fun getCurrentValueFromJsonParser(parser: JsonParser): Any? {
    return when (parser.currentToken) {
        JsonToken.VALUE_TRUE,
        JsonToken.VALUE_FALSE -> parser.booleanValue
        JsonToken.VALUE_NUMBER_INT,
        JsonToken.VALUE_NUMBER_FLOAT -> parser.decimalValue
        JsonToken.FIELD_NAME,
        JsonToken.VALUE_STRING -> parser.text
        JsonToken.VALUE_EMBEDDED_OBJECT -> parser.embeddedObject
        else -> null
    }
}

internal fun getValueFromJsonLogicResult(result: JsonLogicResult): Any? {
    return when (result) {
        is JsonLogicResult.Success -> result.value
        JsonLogicResult.Failure.NullResult -> null
        else -> throw InvalidJsonLogicException("Invalid Json Logic")
    }
}

internal fun getJsonTokenFromValue(value: Any?): JsonToken {
    return when (value) {
        null -> JsonToken.VALUE_NULL
        true -> JsonToken.VALUE_TRUE
        false -> JsonToken.VALUE_FALSE
        is Number -> JsonToken.VALUE_NUMBER_FLOAT
        is String -> JsonToken.VALUE_STRING
        else -> JsonToken.VALUE_EMBEDDED_OBJECT
    }
}

internal fun convertTokensToJsonStruct(tokens: LinkList<MutablePair<JsonToken, Any?>>): JsonStruct {
    val stack = LinkedList<JsonStruct>()
    for (token in tokens) {
        when (token.first) {
            JsonToken.START_OBJECT -> stack.addLast(JsonObject())
            JsonToken.START_ARRAY -> stack.addLast(JsonArray())
            JsonToken.END_OBJECT,
            JsonToken.END_ARRAY -> consolidateLastValueInStack(stack)
            JsonToken.FIELD_NAME -> stack.addLast(JsonField(token.second as String))
            JsonToken.VALUE_STRING,
            JsonToken.VALUE_NUMBER_INT,
            JsonToken.VALUE_NUMBER_FLOAT,
            JsonToken.VALUE_TRUE,
            JsonToken.VALUE_FALSE,
            JsonToken.VALUE_NULL,
            JsonToken.VALUE_EMBEDDED_OBJECT -> {
                stack.addLast(JsonValue(token.second))
                consolidateLastValueInStack(stack)
            }
            else -> throw InvalidJsonLogicException("Invalid json token")
        }
    }
    return stack.removeLast()
}

private fun consolidateLastValueInStack(stack: LinkedList<JsonStruct>) {
    if (stack.size < 2) return
    when (val secondLast = stack[stack.size - 2]) {
        is JsonField -> {
            if (stack.size < 3) throw InvalidJsonLogicException("Invalid Json")
            val thirdLast = stack[stack.size - 3]
            if ((thirdLast is JsonObject).not()) throw InvalidJsonLogicException("Invalid Json")
            (thirdLast as JsonObject).value[secondLast.value] = stack.last.value
            stack.removeLast()
            stack.removeLast()
        }
        is JsonArray -> {
            secondLast.value.addLast(stack.last.value)
            stack.removeLast()
        }
        else -> throw InvalidJsonLogicException("Invalid Json")
    }
}
