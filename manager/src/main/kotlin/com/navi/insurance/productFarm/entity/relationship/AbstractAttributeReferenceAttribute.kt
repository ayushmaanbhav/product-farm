package com.navi.insurance.productFarm.entity.relationship

import com.navi.insurance.productFarm.entity.AbstractEntity
import com.navi.insurance.productFarm.entity.id.AbstractAttributeReferenceAttributeId
import javax.persistence.EmbeddedId
import javax.persistence.Entity
import javax.persistence.Table

@Entity
@Table(name = "abstract_attribute_reference_attribute")
data class AbstractAttributeReferenceAttribute(
    @EmbeddedId val abstractAttributeReferenceAttributeId: AbstractAttributeReferenceAttributeId,
) : AbstractEntity<AbstractAttributeReferenceAttribute>(AbstractAttributeReferenceAttribute::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
