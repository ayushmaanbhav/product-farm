package io.github.ayushmaanbhav.jsonLogic.operations.numeric.compare

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.utils.asList

internal object LessThanOrEqualTo : StandardLogicOperation, RangeComparingOperation {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any =
        compareOrBetween(expression.asList) { first, second -> first <= second }
}
