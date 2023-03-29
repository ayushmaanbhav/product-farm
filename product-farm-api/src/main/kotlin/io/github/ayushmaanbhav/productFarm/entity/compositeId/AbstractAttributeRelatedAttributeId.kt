package io.github.ayushmaanbhav.productFarm.entity.compositeId

import java.io.Serializable
import jakarta.persistence.Embeddable

@Embeddable
data class AbstractAttributeRelatedAttributeId(
    val abstractPath: String,
    val referenceAbstractPath: String,
    val relationship: String,
) : Serializable
