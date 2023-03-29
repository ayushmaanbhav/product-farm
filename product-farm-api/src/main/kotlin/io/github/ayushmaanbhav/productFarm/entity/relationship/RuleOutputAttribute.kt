package io.github.ayushmaanbhav.productFarm.entity.relationship

import io.github.ayushmaanbhav.productFarm.entity.AbstractEntity
import io.github.ayushmaanbhav.productFarm.entity.compositeId.RuleOutputAttributeId
import jakarta.persistence.Column
import jakarta.persistence.EmbeddedId
import jakarta.persistence.Entity
import jakarta.persistence.Table

@Entity
@Table(name = "rule_output_attribute")
data class RuleOutputAttribute(
    @EmbeddedId val id: RuleOutputAttributeId,
    @Column(name = "`order`")
    val order: Int,
) : AbstractEntity<RuleOutputAttribute>(RuleOutputAttribute::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
