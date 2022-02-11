package com.navi.insurance.productFarm.entity.relationship

import com.navi.insurance.productFarm.entity.AbstractEntity
import com.navi.insurance.productFarm.entity.id.AttributeDisplayNameId
import javax.persistence.EmbeddedId
import javax.persistence.Entity
import javax.persistence.Id
import javax.persistence.Index
import javax.persistence.Table

@Entity
@Table(
    name = "product_display_name",
    indexes = [
        Index(columnList = "abstractPath,displayName", unique = true)
    ]
)
data class AttributeDisplayName(
    @EmbeddedId val attributeDisplayNameId: AttributeDisplayNameId,
    val abstractPath: String,
) : AbstractEntity<AttributeDisplayName>(AttributeDisplayName::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
