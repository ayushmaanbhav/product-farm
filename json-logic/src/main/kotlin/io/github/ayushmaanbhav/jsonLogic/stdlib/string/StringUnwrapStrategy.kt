package io.github.ayushmaanbhav.jsonLogic.stdlib.string

import io.github.ayushmaanbhav.jsonLogic.utils.asList

internal interface StringUnwrapStrategy {
    fun unwrapValueAsString(wrappedValue: Any?): String? =
        with(wrappedValue.asList) {
            if (size <= 1) {
                (firstOrNull() as? String)
            } else null
        }
}
