package io.github.ayushmaanbhav.jsonLogic.stdlib

import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
class TestInput(
    val expression: Map<String, Any?>,
    val data: Any? = emptyMap<String, Any>(),
    val result: JsonLogicResult
)
