package io.github.ayushmaanbhav.jsonLogic.operations

import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimal
import io.github.ayushmaanbhav.jsonLogic.utils.toBigDecimalOrNull

internal interface ComparableUnwrapStrategy: BooleanUnwrapStrategy {
    fun unwrapAsComparable(first: Comparable<*>?, second: Comparable<*>?): List<Comparable<*>?>? = when {
        first is Number && second is Number -> listOf(first.toBigDecimal(), second.toBigDecimal())
        first is String && second is Number -> listOf(first.toBigDecimalOrNull(), second.toBigDecimal())
        first is Number && second is String -> listOf(first.toBigDecimal(), second.toBigDecimalOrNull())
        first is Boolean || second is Boolean -> listOf(unwrapValueAsBoolean(first), unwrapValueAsBoolean(second))
        else -> unwrapAsComparableWithTypeSensitivity(first, second)
    }

    fun unwrapAsComparableWithTypeSensitivity(
        first: Comparable<*>?, second: Comparable<*>?
    ): List<Comparable<*>?>? = when {
        (first != null && second != null && first::class == second::class) -> listOf(first, second)
        first == null && second == null -> listOf(first, second)
        else -> null
    }
}
