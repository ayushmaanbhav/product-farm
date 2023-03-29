package io.github.ayushmaanbhav.rule.domain.ruleEngine.model.rule

interface Rule {
    fun getId(): String
    fun ruleType(): String
    fun getInputAttributePaths(): Set<String>
    fun getOutputAttributePaths(): Set<String>
    fun getTags(): Set<String>
    fun getExpression(): String
}
