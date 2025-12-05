package io.github.ayushmaanbhav.ruleEngine

import io.github.ayushmaanbhav.ruleEngine.model.rule.Rule

class RuleImpl(
    private val id: String, private val type: String, private val inpa: Set<String>,
    private val outa: Set<String>, private val tags: Set<String>, private val exp: String
): Rule {
    constructor(id: String, exp: String) : this(id, "", emptySet(), emptySet(), emptySet(), exp)
    constructor(id: String, inpa: Set<String>, outa: Set<String>) : this(id, "", inpa, outa, emptySet(), "")
    constructor(id: String, inpa: Set<String>, outa: Set<String>, tags: Set<String>) : this(id, "", inpa, outa, tags, "")

    constructor(id: String, type: String, inpa: Set<String>, outa: Set<String>, tags: Set<String>) : this(id, type, inpa, outa, tags, "")

    override fun getId(): String = id
    override fun ruleType(): String = type
    override fun getInputAttributePaths(): Set<String> = inpa
    override fun getOutputAttributePaths(): Set<String> = outa
    override fun getTags(): Set<String> = tags
    override fun getExpression(): String = exp

    override fun toString(): String = id
}
