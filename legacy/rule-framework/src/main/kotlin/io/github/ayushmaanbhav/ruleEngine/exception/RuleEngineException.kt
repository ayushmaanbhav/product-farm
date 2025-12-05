package io.github.ayushmaanbhav.ruleEngine.exception

class RuleEngineException : Exception {
    constructor(message: String, cause: Throwable) : super(message, cause)
    constructor(message: String) : super(message)
}
