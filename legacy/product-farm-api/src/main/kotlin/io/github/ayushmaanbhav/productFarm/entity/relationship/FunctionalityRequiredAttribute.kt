package io.github.ayushmaanbhav.productFarm.entity.relationship

import io.github.ayushmaanbhav.productFarm.entity.api.AbstractEntity
import io.github.ayushmaanbhav.productFarm.entity.ProductFunctionality
import io.github.ayushmaanbhav.productFarm.entity.compositeId.FunctionalityRequiredAttributeId
import jakarta.persistence.Column
import jakarta.persistence.EmbeddedId
import jakarta.persistence.Entity
import jakarta.persistence.Table

@Entity
@Table(name = "product_functionality_required_attribute")
data class FunctionalityRequiredAttribute(
    @EmbeddedId val id: FunctionalityRequiredAttributeId,
    val description: String,
    @Column(name = "`order`")
    val order: Int,
) : AbstractEntity<ProductFunctionality>(ProductFunctionality::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
