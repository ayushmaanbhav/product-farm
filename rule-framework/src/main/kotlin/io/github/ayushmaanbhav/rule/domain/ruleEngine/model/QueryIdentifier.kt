package io.github.ayushmaanbhav.rule.domain.ruleEngine.model

data class QueryIdentifier(private val contextIdentifier: String, private val queries: List<Query>)
