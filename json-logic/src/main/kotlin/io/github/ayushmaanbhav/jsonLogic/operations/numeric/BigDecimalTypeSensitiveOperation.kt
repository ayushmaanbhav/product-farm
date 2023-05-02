package io.github.ayushmaanbhav.jsonLogic.operations.numeric

import io.github.ayushmaanbhav.jsonLogic.config.MathContext
import io.github.ayushmaanbhav.jsonLogic.utils.asBigDecimalList
import java.math.BigDecimal

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
