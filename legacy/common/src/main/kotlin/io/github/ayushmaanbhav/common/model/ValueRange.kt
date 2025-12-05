package io.github.ayushmaanbhav.common.model

import io.github.ayushmaanbhav.common.model.constant.ValueRangeType
import java.util.*

data class ValueRange<T : Comparable<T>>(val values: SortedSet<T>, val type: ValueRangeType) {
    init {
        when (type) {
            ValueRangeType.DISCRETE_VALUES -> require(values.size > 0) { "Values size should be > 0 when type is DISCREET_VALUES" }
            ValueRangeType.RANGE -> require(values.size == 2) { "Values size should be 2 when type is RANGE" }
        }
    }

    fun inRange(value: T): Boolean {
        return when (type) {
            ValueRangeType.DISCRETE_VALUES -> values.contains(value)
            ValueRangeType.RANGE -> values.first() <= value && values.last() >= value
        }
    }
}