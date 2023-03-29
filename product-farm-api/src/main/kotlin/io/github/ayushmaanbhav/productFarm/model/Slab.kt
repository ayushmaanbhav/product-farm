package io.github.ayushmaanbhav.productFarm.model

import com.fasterxml.jackson.annotation.JsonProperty
import com.fasterxml.jackson.databind.JsonNode

data class Slab(
    @JsonProperty("cases")
    val cases: List<Case>,
    val commonExpression: String?,
    @JsonProperty("defaultReturn")
    val defaultReturnObject: List<JsonNode>?,
)