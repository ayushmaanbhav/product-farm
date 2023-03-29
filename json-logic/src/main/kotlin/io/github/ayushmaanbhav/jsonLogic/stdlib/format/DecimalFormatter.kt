package io.github.ayushmaanbhav.jsonLogic.stdlib.format

import java.math.BigDecimal
import io.github.ayushmaanbhav.jsonLogic.utils.asList
import io.github.ayushmaanbhav.jsonLogic.utils.secondOrNull
import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimalOrNull

internal interface DecimalFormatter {
    fun formatDecimal(
        expression: Any?,
        data: Any?,
        formatFloatingPoint: (format: String, arg: BigDecimal) -> String
    ): String? {
        return with(expression.asList) {
            val format = firstOrNull().toString()
            val formattedArgument = secondOrNull().toString()

            runCatching { format.formatAsFloatingDecimal(formattedArgument, formatFloatingPoint) }
                .fold(
                    onSuccess = { it },
                    onFailure = { null }
                )
        }
    }

    private fun String.formatAsFloatingDecimal(
        formattedArgument: String,
        formatFloatingPoint: (String, BigDecimal) -> String
    ) = if (matches("%[\\d|.]*[f]".toRegex())) {
            formattedArgument.toBigDecimalOrNull()?.let {
                formatFloatingPoint(this, it)
            }
        } else {
            null
        }
}
