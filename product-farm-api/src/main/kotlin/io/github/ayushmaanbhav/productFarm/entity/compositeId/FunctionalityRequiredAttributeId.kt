package io.github.ayushmaanbhav.productFarm.entity.compositeId

import jakarta.persistence.Embeddable
import java.io.Serializable

@Embeddable
data class FunctionalityRequiredAttributeId(
    val functionalityId: String,
    val abstractPath: String,
) : Serializable
