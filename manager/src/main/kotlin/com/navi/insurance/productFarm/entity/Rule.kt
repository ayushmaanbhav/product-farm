package com.navi.insurance.productFarm.entity

import com.navi.insurance.productFarm.entity.relationship.RuleInputAttribute
import com.navi.insurance.productFarm.entity.relationship.RuleOutputAttribute
import org.hibernate.annotations.NaturalId
import javax.persistence.CascadeType
import javax.persistence.Entity
import javax.persistence.FetchType
import javax.persistence.Id
import javax.persistence.JoinColumn
import javax.persistence.OneToMany
import javax.persistence.Table

@Entity
@Table(name = "rule")
data class Rule(
    @Id @NaturalId val id: String,
    val type: String,
    @OneToMany(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.EAGER)
    @JoinColumn(name = "ruleId", referencedColumnName = "id")
    val inputAttribute: Set<RuleInputAttribute>,
    @OneToMany(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.EAGER)
    @JoinColumn(name = "ruleId", referencedColumnName = "id")
    val outputAttribute: Set<RuleOutputAttribute>,
    val displayExpression: String,
    val displayExpressionVersion: String,
    val expression: String,
    val description: String?,
) : AbstractEntity<Rule>(Rule::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
