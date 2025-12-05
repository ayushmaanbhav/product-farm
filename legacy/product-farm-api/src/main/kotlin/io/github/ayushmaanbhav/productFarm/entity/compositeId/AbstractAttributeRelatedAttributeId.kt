package io.github.ayushmaanbhav.productFarm.entity.compositeId

import jakarta.persistence.Embeddable
import java.io.Serializable

@Embeddable
data class AbstractAttributeRelatedAttributeId(
    val abstractPath: String,
    val referenceAbstractPath: String,
    val relationship: String,
) : Serializable
