package com.navi.insurance.productFarm.entity.id

import java.io.Serializable
import javax.persistence.Embeddable

@Embeddable
data class FunctionalityRequiredAttributeId(
    val functionalityId: String,
    val abstractPath: String,
) : Serializable
