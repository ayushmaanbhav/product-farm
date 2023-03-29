package io.github.ayushmaanbhav.productFarm.entity.relationship

import io.github.ayushmaanbhav.productFarm.entity.AbstractEntity
import io.github.ayushmaanbhav.productFarm.entity.compositeId.AbstractAttributeRelatedAttributeId
import jakarta.persistence.Column
import jakarta.persistence.EmbeddedId
import jakarta.persistence.Entity
import jakarta.persistence.Table

@Entity
@Table(name = "abstract_attribute_related_attribute")
data class AbstractAttributeRelatedAttribute(
    @EmbeddedId val id: AbstractAttributeRelatedAttributeId,
    @Column(name = "`order`")
    val order: Int,
) : AbstractEntity<AbstractAttributeRelatedAttribute>(AbstractAttributeRelatedAttribute::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
