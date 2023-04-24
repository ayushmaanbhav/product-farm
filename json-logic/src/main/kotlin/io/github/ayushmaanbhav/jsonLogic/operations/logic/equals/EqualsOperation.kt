package io.github.ayushmaanbhav.jsonLogic.operations.logic.equals

import io.github.ayushmaanbhav.jsonLogic.operations.ComparingOperation
import io.github.ayushmaanbhav.jsonLogic.operations.logic.unwrap.EqualsUnwrapStrategy
import io.github.ayushmaanbhav.jsonLogic.operations.logic.unwrap.SingleNestedValueUnwrapStrategy
import io.github.ayushmaanbhav.jsonLogic.utils.asList
import io.github.ayushmaanbhav.jsonLogic.utils.secondOrNull

internal interface EqualsOperation : ComparingOperation, EqualsUnwrapStrategy, SingleNestedValueUnwrapStrategy {
    fun compare(values: Any?, operator: (Int, Int) -> Boolean): Boolean =
        with(values.asList) {
            val firstUnwrappedValue = unwrapSingleNestedValueOrDefault(firstOrNull())
            val secondUnwrappedValue = unwrapSingleNestedValueOrDefault(secondOrNull())
            val firstPossibleTrueValues = EqualsTableOfTruth[firstUnwrappedValue]
            val secondPossibleTrueValues = EqualsTableOfTruth[secondUnwrappedValue]

            if (firstPossibleTrueValues != null || secondPossibleTrueValues != null) {
                firstPossibleTrueValues?.contains(secondUnwrappedValue) ?: false
                    || secondPossibleTrueValues?.contains(firstUnwrappedValue) ?: false
            } else {
                compareListOfTwo(map(::unwrapValue), operator)
            }
        }
}
