package com.navi.insurance.productFarm.entity.id

import java.io.Serializable
import javax.persistence.Embeddable

@Embeddable
data class AbstractAttributeReferenceAttributeId(
    val abstractPath: String,
    val referenceAbstractPath: String,
) : Serializable
