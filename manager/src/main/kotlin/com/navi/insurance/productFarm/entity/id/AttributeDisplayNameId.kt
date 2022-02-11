package com.navi.insurance.productFarm.entity.id

import java.io.Serializable
import javax.persistence.Embeddable

@Embeddable
data class AttributeDisplayNameId(
    val productId: String,
    val displayName: String,
) : Serializable
