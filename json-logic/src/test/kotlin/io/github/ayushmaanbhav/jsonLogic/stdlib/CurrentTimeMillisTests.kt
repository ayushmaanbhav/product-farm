package io.github.ayushmaanbhav.jsonLogic.stdlib

import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.kotest.core.spec.style.FunSpec
import io.kotest.matchers.should
import io.kotest.matchers.types.beInstanceOf

class CurrentTimeMillisTests : FunSpec() {
    init {
        val operatorName = "currentTime"
        val logicEngine = JsonLogicEngine.Builder()
            .addStandardOperation(operatorName, CurrentTimeMillis)
            .build()

        test("CurrentTimeMillis.evaluateLogic should be Long type") {
            val result = logicEngine.evaluate(mapOf(operatorName to emptyList<Any>()), null)

            result should beInstanceOf<JsonLogicResult.Success>()
            (result as JsonLogicResult.Success).value should beInstanceOf<Long>()
        }
    }
}
