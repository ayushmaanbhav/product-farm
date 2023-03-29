package io.github.ayushmaanbhav.jsonLogic.operations.string

import java.math.BigDecimal
import io.github.ayushmaanbhav.jsonLogic.utils.asList
import io.github.ayushmaanbhav.jsonLogic.utils.isWhole
import io.github.ayushmaanbhav.jsonLogic.utils.toStringOrEmpty

internal interface StringUnwrapStrategy {
    fun unwrapValueAsString(wrappedValue: Any?): List<String> = wrappedValue.asList.map(::stringify)

    private fun stringify(value: Any?) = (value as? List<*>)?.flatMap { nestedValue ->
        nestedValue.flattenNestedLists()
    }?.joinToString(separator = ",") ?: value.formatAsString()

    private fun Any?.flattenNestedLists(): List<String> = (this as? List<*>)?.flatMap {
        it.flattenNestedLists()
    } ?: listOf(formatAsString())

    private fun Any?.formatAsString(): String =
        if (this is Number && isWhole()) {
            toInt().toString()
        } else if (this is BigDecimal) {
            this.toPlainString()
        } else toStringOrEmpty()
}
