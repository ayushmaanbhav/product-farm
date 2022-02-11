package com.navi.insurance.productFarm.entity.relationship

import com.navi.insurance.productFarm.entity.AbstractEntity
import com.navi.insurance.productFarm.entity.id.RuleInputAttributeId
import javax.persistence.EmbeddedId
import javax.persistence.Entity
import javax.persistence.Table

@Entity
@Table(name = "rule_input_attribute")
data class RuleInputAttribute(
    @EmbeddedId val ruleInputAttributeId: RuleInputAttributeId,
) : AbstractEntity<RuleInputAttribute>(RuleInputAttribute::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
