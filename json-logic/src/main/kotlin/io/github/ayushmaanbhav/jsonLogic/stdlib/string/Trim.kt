package io.github.ayushmaanbhav.jsonLogic.stdlib.string

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.utils.asList

object Trim : StandardLogicOperation, StringUnwrapStrategy {
    private const val TEXT_ARG_INDEX = 0
    private const val CHAR_ARG_INDEX = 1
    private const val MODE_ARG_INDEX = 2

    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? =
        expression.asList.toOperationArguments()?.invokeTrim()

    private fun List<Any?>.toOperationArguments(): TrimArguments? = runCatching {
        TrimArguments(
            text = get(TEXT_ARG_INDEX) as String,
            char = (get(CHAR_ARG_INDEX) as String).single(),
            mode = (get(MODE_ARG_INDEX) as String).toTrimMode()
        )
    }.fold(
        onSuccess = { it },
        onFailure = { null }
    )

    private fun String?.toTrimMode() = when (this) {
        "start" -> TrimMode.Start
        "end" -> TrimMode.End
        "bothEnds" -> TrimMode.BothEnds
        else -> throw IllegalStateException("Invalid TrimMode value")
    }

    private fun TrimArguments.invokeTrim() = when (mode) {
        TrimMode.Start -> this.text.trimStart(char)
        TrimMode.End -> this.text.trimEnd(char)
        TrimMode.BothEnds -> this.text.trim(char)
    }
}

private data class TrimArguments(
    val text: String,
    val char: Char,
    val mode: TrimMode
)

private sealed class TrimMode {
    object Start : TrimMode()
    object End : TrimMode()
    object BothEnds : TrimMode()
}
