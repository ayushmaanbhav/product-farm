package io.github.ayushmaanbhav.jsonLogic.operations

import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimal
import java.math.BigInteger

internal interface BooleanUnwrapStrategy {
    fun unwrapValueAsBoolean(wrappedValue: Any?): Boolean? = when (wrappedValue) {
        is Boolean -> wrappedValue
        is Number -> wrappedValue.toBigDecimal().toBigInteger() > BigInteger.ZERO
        is String -> wrappedValue.toBigDecimalOrNull()?.let { it.toBigInteger() > BigInteger.ZERO }
        else -> null
    }
}
