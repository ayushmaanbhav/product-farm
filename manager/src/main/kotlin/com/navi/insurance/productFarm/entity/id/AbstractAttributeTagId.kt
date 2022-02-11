package com.navi.insurance.productFarm.entity.id

import java.io.Serializable
import javax.persistence.Embeddable

@Embeddable
data class AbstractAttributeTagId(
    val abstractPath: String,
    val tag: String,
) : Serializable
