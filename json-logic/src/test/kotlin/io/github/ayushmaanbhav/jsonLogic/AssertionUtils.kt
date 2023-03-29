package io.github.ayushmaanbhav.jsonLogic

import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.kotest.matchers.shouldBe

infix fun JsonLogicResult.valueShouldBe(other: JsonLogicResult) {
    if(this is JsonLogicResult.Success && other is JsonLogicResult.Success) {
        value shouldBe other.value
    } else {
        this shouldBe other
    }
}
