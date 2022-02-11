package com.navi.insurance.productFarm.entity.relationship

import com.navi.insurance.productFarm.entity.AbstractEntity
import com.navi.insurance.productFarm.entity.ProductFunctionality
import com.navi.insurance.productFarm.entity.id.FunctionalityRequiredAttributeId
import javax.persistence.EmbeddedId
import javax.persistence.Entity
import javax.persistence.Table

@Entity
@Table(name = "product_functionality_required_attribute")
data class FunctionalityRequiredAttribute(
    @EmbeddedId val functionalityRequiredAttributeId: FunctionalityRequiredAttributeId,
    val description: String,
) : AbstractEntity<ProductFunctionality>(ProductFunctionality::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
