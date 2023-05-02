package io.github.ayushmaanbhav.productFarm.entity.compositeId

import jakarta.persistence.Embeddable
import java.io.Serializable

@Embeddable
data class RuleOutputAttributeId(
    val ruleId: String,
    val path: String,
) : Serializable
