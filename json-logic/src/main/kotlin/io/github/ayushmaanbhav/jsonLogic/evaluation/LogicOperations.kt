package io.github.ayushmaanbhav.jsonLogic.evaluation

import io.github.ayushmaanbhav.jsonLogic.api.operation.FunctionalLogicOperation
import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation

internal data class LogicOperations(
    val standardOperations: Map<String, StandardLogicOperation> = emptyMap(),
    val functionalOperations: Map<String, FunctionalLogicOperation> = emptyMap()
)
