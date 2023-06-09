package io.github.ayushmaanbhav.jsonLogic.operations.data

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.operations.data.unwrap.ValueFetchingUnwrapStrategy
import io.github.ayushmaanbhav.jsonLogic.utils.asList
import io.github.ayushmaanbhav.jsonLogic.utils.intOrZero
import io.github.ayushmaanbhav.jsonLogic.utils.secondOrNull

internal object Var : StandardLogicOperation, ValueFetchingUnwrapStrategy {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? =
        unwrapDataKeys(expression.asList, config.nestedVariablePathDelimiter)?.fetchValueOrDefault( expression, data)

    private fun List<String>.fetchValueOrDefault(expression: Any?, data: Any?): Any? {
        val value = if (isNotEmpty()) {
            getIndexedValue(data, this)
        } else {
            data
        }

        return if (shouldUseDefaultValue(value, expression)) {
            (expression as? List<*>)?.secondOrNull()
        } else {
            value
        }
    }

    private fun getIndexedValue(value: Any?, indexParts: List<String>): Any? {
        return when (value) {
            is List<*> -> {
                if (indexParts.size == 1) {
                    value[indexParts.first().intOrZero]
                } else {
                    getRecursive(indexParts, value)
                }
            }
            is Map<*, *> -> {
                val initial = value[indexParts.first()]
                indexParts.drop(1).fold(initial) { acc: Any?, indexPart: String ->
                    (acc as? Map<*, *>)?.get(indexPart)
                }
            }
            else -> value
        }
    }

    private fun shouldUseDefaultValue(value: Any?, expression: Any?) = (value == expression || value == null)
        && expression is List<*>
        && expression.size > 1

    private tailrec fun getRecursive(indexes: List<String>, data: List<Any?>): Any? = indexes.firstOrNull()?.apply {
        val indexedData = data.getOrNull(intOrZero)
        return if (indexedData is List<*>) {
            getRecursive(indexes.subList(1, indexes.size), indexedData)
        } else {
            data.getOrNull(intOrZero)
        }
    }
}
