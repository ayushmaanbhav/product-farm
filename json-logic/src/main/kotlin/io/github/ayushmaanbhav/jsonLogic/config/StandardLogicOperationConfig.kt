package io.github.ayushmaanbhav.jsonLogic.config

data class StandardLogicOperationConfig(val mathContext: MathContext, val nestedVariablePathDelimiter: Char) {
    constructor(mathContext: MathContext) : this(mathContext, DEFAULT_NESTED_VARIABLE_PATH_DELIMITER)

    companion object {
        const val DEFAULT_NESTED_VARIABLE_PATH_DELIMITER = '.'
    }
}
