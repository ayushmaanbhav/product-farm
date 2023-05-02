package io.github.ayushmaanbhav.productFarm.entity.compositeId

import jakarta.persistence.Embeddable
import java.io.Serializable

@Embeddable
data class AbstractAttributeTagId(
    val abstractPath: String,
    val tag: String,
) : Serializable
