package com.navi.insurance.productFarm.entity.relationship

import com.navi.insurance.productFarm.entity.AbstractEntity
import com.navi.insurance.productFarm.entity.id.RuleOutputAttributeId
import javax.persistence.EmbeddedId
import javax.persistence.Entity
import javax.persistence.Table

@Entity
@Table(name = "rule_output_attribute")
data class RuleOutputAttribute(
    @EmbeddedId val ruleOutputAttributeId: RuleOutputAttributeId,
) : AbstractEntity<RuleOutputAttribute>(RuleOutputAttribute::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
