package io.github.ayushmaanbhav.jsonLogic.operations.numeric

import java.math.BigDecimal
import io.github.ayushmaanbhav.jsonLogic.config.MathContext
import io.github.ayushmaanbhav.jsonLogic.utils.asBigDecimalList

internal interface BigDecimalTypeSensitiveOperation {
    fun bigDecimalResultOrNull(
        mc: MathContext, expression: Any?, operation: (List<BigDecimal>) -> BigDecimal?
    ): BigDecimal? {
        val elements = expression?.asBigDecimalList(mc)
        val notNullElements = elements?.filterNotNull()
        return if (notNullElements?.size == elements?.size) {
            elements?.filterNotNull()?.let { operation(it) }
        } else null
    }
}
