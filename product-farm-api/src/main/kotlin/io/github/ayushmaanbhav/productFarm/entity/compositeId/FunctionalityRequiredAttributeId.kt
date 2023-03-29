package io.github.ayushmaanbhav.productFarm.entity.compositeId

import java.io.Serializable
import jakarta.persistence.Embeddable

@Embeddable
data class FunctionalityRequiredAttributeId(
    val functionalityId: String,
    val abstractPath: String,
) : Serializable
