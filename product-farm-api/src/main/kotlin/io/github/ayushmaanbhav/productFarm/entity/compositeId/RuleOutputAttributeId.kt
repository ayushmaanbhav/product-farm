package io.github.ayushmaanbhav.productFarm.entity.compositeId

import java.io.Serializable
import jakarta.persistence.Embeddable

@Embeddable
data class RuleOutputAttributeId(
    val ruleId: String,
    val path: String,
) : Serializable
