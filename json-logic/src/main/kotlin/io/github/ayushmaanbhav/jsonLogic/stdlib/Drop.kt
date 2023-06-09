package io.github.ayushmaanbhav.jsonLogic.stdlib

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.utils.asList
import io.github.ayushmaanbhav.jsonLogic.utils.secondOrNull
import io.github.ayushmaanbhav.jsonLogic.utils.thirdOrNull

object Drop : StandardLogicOperation {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? =
        with(expression.asList) {
            val dropCandidate = firstOrNull()
            val count = secondOrNull()
            val mode = (thirdOrNull() as? String).toDropMode()

            (count as? Int)?.let { dropCandidate.dropElements(it, mode) }
        }

    private fun String?.toDropMode() = when (this) {
        "first" -> DropMode.First
        "last" -> DropMode.Last
        else -> DropMode.Unknown
    }

    private fun Any?.dropElements(count: Int, mode: DropMode) =
        when (this) {
            is String -> modeBasedDrop(mode = mode, first = { drop(count) }, last = { dropLast(count) })
            is List<*> -> modeBasedDrop(mode = mode, first = { drop(count) }, last = { dropLast(count) })
            else -> null
        }

    private fun modeBasedDrop(mode: DropMode, first: (() -> Any?), last: (() -> Any?)) =
        when (mode) {
            DropMode.First -> first()
            DropMode.Last -> last()
            DropMode.Unknown -> null
        }
}

private sealed class DropMode {
    object First : DropMode()
    object Last : DropMode()
    object Unknown : DropMode()
}
