package com.navi.insurance.productFarm.dto

data class RuleDto(
    val type: String,
    val inputAttribute: Set<String>,
    val outputAttribute: Set<String>,
    val displayExpression: String,
    val displayExpressionVersion: String,
    val expression: String,
    val description: String?,
)
