package io.github.ayushmaanbhav.jsonLogic.utils.type

import java.util.*

internal sealed class JsonStruct(open val value: Any?)

internal data class JsonObject(override val value: LinkedHashMap<String, Any?> = LinkedHashMap()): JsonStruct(value)

internal data class JsonArray(override val value: LinkedList<Any?> = LinkedList()): JsonStruct(value)

internal data class JsonField(override val value: String): JsonStruct(value)

internal data class JsonValue(override val value: Any?): JsonStruct(value)
