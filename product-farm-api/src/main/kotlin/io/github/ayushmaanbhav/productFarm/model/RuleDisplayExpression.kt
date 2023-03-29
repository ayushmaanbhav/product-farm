package io.github.ayushmaanbhav.productFarm.model

import com.fasterxml.jackson.annotation.JsonProperty
import com.fasterxml.jackson.databind.JsonNode

data class RuleDisplayExpression(
    val slab: Slab?,
    val expression: String?,
    @JsonProperty("return")
    val returnObject: List<JsonNode>?,
)
