package com.navi.insurance.productFarm.entity.relationship

import com.navi.insurance.productFarm.entity.AbstractEntity
import com.navi.insurance.productFarm.entity.id.AbstractAttributeTagId
import javax.persistence.EmbeddedId
import javax.persistence.Entity
import javax.persistence.Index
import javax.persistence.Table

@Entity
@Table(
    name = "product_display_name",
    indexes = [
        Index(columnList = "productId")
    ]
)
data class AbstractAttributeTag(
    @EmbeddedId val abstractAttributeTagId: AbstractAttributeTagId,
    val productId: String,
) : AbstractEntity<AbstractAttributeTag>(AbstractAttributeTag::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
