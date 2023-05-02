package io.github.ayushmaanbhav.productFarm.entity.compositeId

import jakarta.persistence.Embeddable
import java.io.Serializable

@Embeddable
data class RuleInputAttributeId(
    val ruleId: String,
    val path: String,
) : Serializable
