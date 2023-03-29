package io.github.ayushmaanbhav.productFarm.entity.compositeId

import java.io.Serializable
import jakarta.persistence.Embeddable

@Embeddable
data class AbstractAttributeTagId(
    val abstractPath: String,
    val tag: String,
) : Serializable
