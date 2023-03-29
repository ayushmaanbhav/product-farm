package io.github.ayushmaanbhav.jsonLogic.operations.numeric.unwrap

import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.config.MathContext
import io.github.ayushmaanbhav.jsonLogic.utils.asList
import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimal
import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimalOrNull

internal interface StrictUnwrapStrategy {
    fun unwrapValue(config: StandardLogicOperationConfig, wrappedValue: Any?): List<Any?> =
        wrappedValue.asList.map { unwrap(config.mathContext, it) }

    private tailrec fun unwrap(mc: MathContext, value: Any?): Any? =
        when (value) {
            is Number -> value.toBigDecimal(mc)
            is String -> value.toBigDecimalOrNull(mc)
            is List<*> -> unwrap(mc, value.firstOrNull())
            else -> null
        }
}
