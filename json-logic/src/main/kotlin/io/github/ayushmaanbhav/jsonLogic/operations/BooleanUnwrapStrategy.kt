package io.github.ayushmaanbhav.jsonLogic.operations

import java.math.BigInteger
import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimal

internal interface BooleanUnwrapStrategy {
    fun unwrapValueAsBoolean(wrappedValue: Any?): Boolean? = when (wrappedValue) {
        is Boolean -> wrappedValue
        is Number -> wrappedValue.toBigDecimal().toBigInteger() > BigInteger.ZERO
        is String -> wrappedValue.toBigDecimalOrNull()?.let { it.toBigInteger() > BigInteger.ZERO }
        else -> null
    }
}
