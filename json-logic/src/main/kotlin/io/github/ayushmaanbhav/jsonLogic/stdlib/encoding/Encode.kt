package io.github.ayushmaanbhav.jsonLogic.stdlib.encoding

import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.stdlib.string.StringUnwrapStrategy
import java.net.URLEncoder

object Encode : StandardLogicOperation, StringUnwrapStrategy {
    override fun evaluateLogic(config: StandardLogicOperationConfig, expression: Any?, data: Any?): Any? {
        return  unwrapValueAsString(expression)?.let {  URLEncoder.encode(it, Charsets.UTF_8.name()) }
    }
}
