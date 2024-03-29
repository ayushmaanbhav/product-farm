package io.github.ayushmaanbhav.productFarm.entity.relationship

import io.github.ayushmaanbhav.productFarm.entity.api.AbstractEntity
import io.github.ayushmaanbhav.productFarm.entity.compositeId.AbstractAttributeTagId
import jakarta.persistence.Column
import jakarta.persistence.EmbeddedId
import jakarta.persistence.Entity
import jakarta.persistence.Index
import jakarta.persistence.Table

@Entity
@Table(
    name = "abstract_attribute_tag",
    indexes = [
        Index(columnList = "productId,tag")
    ],
)
data class AbstractAttributeTag(
    @EmbeddedId val id: AbstractAttributeTagId,
    val productId: String,
    @Column(name = "`order`")
    val order: Int,
) : AbstractEntity<AbstractAttributeTag>(AbstractAttributeTag::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
