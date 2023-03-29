package io.github.ayushmaanbhav.productFarm.entity.compositeId

import java.io.Serializable
import jakarta.persistence.Embeddable

@Embeddable
data class AttributeDisplayNameId(
    val productId: String,
    val displayName: String,
) : Serializable
