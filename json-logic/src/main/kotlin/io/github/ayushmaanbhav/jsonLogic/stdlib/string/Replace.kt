package io.github.ayushmaanbhav.jsonLogic.stdlib.string

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import kotlin.runCatching
import io.github.ayushmaanbhav.jsonLogic.utils.asList

object Replace: StandardLogicOperation, StringUnwrapStrategy {
    private const val REPLACE_CANDIDATE_INDEX = 0
    private const val OLD_STRING_INDEX = 1
    private const val NEW_STRING_INDEX = 2
    private const val MODE_INDEX = 3

    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? =
        expression.asList.runCatching {
            val replaceData = ReplaceData(
                get(REPLACE_CANDIDATE_INDEX) as String,
                get(OLD_STRING_INDEX) as String,
                get(NEW_STRING_INDEX) as String,
            )
            val mode = ReplaceMode.from(get(MODE_INDEX) as String, replaceData)

            mode()
        }.fold(
            onSuccess = { it },
            onFailure = { null }
        )
}
