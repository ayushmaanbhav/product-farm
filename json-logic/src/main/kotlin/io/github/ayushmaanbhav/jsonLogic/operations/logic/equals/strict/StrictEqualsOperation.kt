package io.github.ayushmaanbhav.jsonLogic.operations.logic.equals.strict

import io.github.ayushmaanbhav.jsonLogic.operations.logic.equals.EqualsOperation
import io.github.ayushmaanbhav.jsonLogic.utils.asList

internal interface StrictEqualsOperation : EqualsOperation {
    override fun compare(values: Any?, operator: (Int, Int) -> Boolean): Boolean {
        return with(values.asList) {
            if (size != 1) {
                compareListOfTwo(map(::unwrapValue), operator)
            } else false
        }
    }

    override fun unwrapValue(wrappedValue: Any?): Any? = (wrappedValue as? Number)?.toDouble() ?: wrappedValue

    override fun unwrapAsComparable(first: Comparable<*>?, second: Comparable<*>?): List<Comparable<*>?>? =
        unwrapAsComparableWithTypeSensitivity(first, second)
}
