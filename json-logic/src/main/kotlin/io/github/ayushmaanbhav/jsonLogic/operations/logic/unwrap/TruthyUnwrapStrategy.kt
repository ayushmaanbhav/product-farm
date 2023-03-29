package io.github.ayushmaanbhav.jsonLogic.operations.logic.unwrap

import java.math.BigDecimal
import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimal

internal interface TruthyUnwrapStrategy {
    fun unwrapValueAsBoolean(wrappedValue: Any?): Boolean = when (wrappedValue) {
        null -> false
        is Boolean -> wrappedValue
        is Number -> BigDecimal.ZERO.compareTo(wrappedValue.toBigDecimal()) != 0
        is String -> wrappedValue.isNotEmpty() && wrappedValue != "[]" && wrappedValue != "null"
        is Collection<*> -> wrappedValue.isNotEmpty()
        is Array<*> -> wrappedValue.isNotEmpty()
        else -> true
    }
}
