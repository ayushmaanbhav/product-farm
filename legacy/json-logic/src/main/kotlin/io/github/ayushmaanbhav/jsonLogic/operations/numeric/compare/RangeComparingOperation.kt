package io.github.ayushmaanbhav.jsonLogic.operations.numeric.compare

import io.github.ayushmaanbhav.jsonLogic.operations.ComparingOperation
import io.github.ayushmaanbhav.jsonLogic.utils.comparableList
import io.github.ayushmaanbhav.jsonLogic.utils.secondOrNull
import io.github.ayushmaanbhav.jsonLogic.utils.thirdOrNull

internal interface RangeComparingOperation : ComparingOperation {
    fun compareOrBetween(
        values: List<Any?>?,
        operator: (Int, Int) -> Boolean
    ) = values?.comparableList?.let { comparableList ->
        when {
            comparableList.size == 2 -> compareListOfTwo(comparableList, operator)
            comparableList.size > 2 -> comparableList.between(operator)
            else -> false
        }
    } ?: false

    private fun List<Comparable<*>?>.between(operator: (Int, Int) -> Boolean): Boolean {
        val firstEvaluation = compareListOfTwo(listOf(firstOrNull(), secondOrNull()), operator)
        val secondEvaluation = compareListOfTwo(listOf(secondOrNull(), thirdOrNull()), operator)
        return firstEvaluation && secondEvaluation
    }
}
