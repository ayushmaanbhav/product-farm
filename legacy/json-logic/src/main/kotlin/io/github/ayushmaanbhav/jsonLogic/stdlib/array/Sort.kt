package io.github.ayushmaanbhav.jsonLogic.stdlib.array

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.utils.asBigDecimalList
import io.github.ayushmaanbhav.jsonLogic.utils.asList
import io.github.ayushmaanbhav.jsonLogic.utils.secondOrNull

object Sort : StandardLogicOperation {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? =
        with(expression.asList) {
            (firstOrNull() as? List<*>)?.let { elementsToSort ->
                val sortingMode = (secondOrNull() as? String).toSortOrder()
                elementsToSort.sortByMode(sortingMode)
            }
        }

    private fun String?.toSortOrder() = when (this) {
        "desc" -> SortOrder.Descending
        "asc" -> SortOrder.Ascending
        else -> SortOrder.Unknown
    }

    private fun List<Any?>.sortByMode(sortingMode: SortOrder) = when {
        containsOnlyElementsOfType<String>() -> this.castAndSortComparable<String>(sortingMode)
        containsOnlyElementsOfType<Boolean>() -> this.castAndSortComparable<Boolean>(sortingMode)
        containsOnlyElementsOfType<Number>() -> asBigDecimalList.filterNotNull().sortComparable(sortingMode)
        else -> null
    }

    private inline fun <reified T> List<Any?>?.containsOnlyElementsOfType() =
        this?.filterIsInstance<T>()?.size == this?.size

    @Suppress("UNCHECKED_CAST")
    private inline fun <reified T : Comparable<T>> List<*>.castAndSortComparable(sortingMode: SortOrder) =
        (this as? List<T>)?.sortComparable(sortingMode)

    private inline fun <reified T : Comparable<T>> List<T>.sortComparable(sortingMode: SortOrder) =
        modeBasedSort(sortingMode = sortingMode, ascSort = { sorted() }, descSort = { sortedDescending() })

    private fun modeBasedSort(sortingMode: SortOrder, ascSort: (() -> Any?), descSort: (() -> Any?)) =
        when (sortingMode) {
            SortOrder.Descending -> descSort()
            SortOrder.Ascending -> ascSort()
            SortOrder.Unknown -> null
        }
}

private sealed class SortOrder {
    object Descending : SortOrder()
    object Ascending : SortOrder()
    object Unknown : SortOrder()
}
