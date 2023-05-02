package io.github.ayushmaanbhav.productFarm.entity.compositeId

import jakarta.persistence.Embeddable
import java.io.Serializable

@Embeddable
data class AttributeDisplayNameId(
    val productId: String,
    val displayName: String,
) : Serializable
