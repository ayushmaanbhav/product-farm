package io.github.ayushmaanbhav.jsonLogic

import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult

data class TestInput(
    val expression: Map<String, Any?>,
    val data: Any? = emptyMap<String, Any>(),
    val result: JsonLogicResult
)
