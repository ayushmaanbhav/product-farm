package io.github.ayushmaanbhav.rule.domain.ruleEngine.model.rule

import java.net.URI

interface Rule {
    fun getId(): String
    fun ruleType(): String
    fun getInputAttributePaths(): Set<String>
    fun getOutputAttributePaths(): Set<String>
    fun getTags(): Set<String>
    fun getExpression(): String
    fun getExpressionUri(): URI
}
