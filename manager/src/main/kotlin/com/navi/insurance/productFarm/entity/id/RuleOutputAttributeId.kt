package com.navi.insurance.productFarm.entity.id

import java.io.Serializable
import javax.persistence.Embeddable

@Embeddable
data class RuleOutputAttributeId(
    val ruleId: String,
    val path: String,
) : Serializable
