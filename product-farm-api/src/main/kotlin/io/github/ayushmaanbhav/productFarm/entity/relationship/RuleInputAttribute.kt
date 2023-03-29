package io.github.ayushmaanbhav.productFarm.entity.relationship

import io.github.ayushmaanbhav.productFarm.entity.AbstractEntity
import io.github.ayushmaanbhav.productFarm.entity.compositeId.RuleInputAttributeId
import jakarta.persistence.Column
import jakarta.persistence.EmbeddedId
import jakarta.persistence.Entity
import jakarta.persistence.Table

@Entity
@Table(name = "rule_input_attribute")
data class RuleInputAttribute(
    @EmbeddedId val id: RuleInputAttributeId,
    @Column(name = "`order`")
    val order: Int,
) : AbstractEntity<RuleInputAttribute>(RuleInputAttribute::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
