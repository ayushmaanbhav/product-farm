package io.github.ayushmaanbhav.productFarm.entity

import ValidRule
import io.github.ayushmaanbhav.productFarm.entity.relationship.RuleInputAttribute
import io.github.ayushmaanbhav.productFarm.entity.relationship.RuleOutputAttribute
import org.hibernate.annotations.NaturalId
import jakarta.persistence.CascadeType
import jakarta.persistence.Entity
import jakarta.persistence.FetchType
import jakarta.persistence.Id
import jakarta.persistence.JoinColumn
import jakarta.persistence.OneToMany
import jakarta.persistence.OrderBy
import jakarta.persistence.Table

@Entity
@Table(name = "rule")
@ValidRule
data class Rule(
    @Id @NaturalId val id: String,
    val type: String,
    @OneToMany(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.LAZY)
    @JoinColumn(name = "ruleId", referencedColumnName = "id")
    @OrderBy("`order`")
    val inputAttributes: List<RuleInputAttribute>,
    @OneToMany(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.LAZY)
    @JoinColumn(name = "ruleId", referencedColumnName = "id")
    @OrderBy("`order`")
    val outputAttributes: List<RuleOutputAttribute>,
    val displayExpression: String,
    val displayExpressionVersion: String,
    val compiledExpression: String,
    val description: String?,
) : AbstractEntity<Rule>(Rule::class), io.github.ayushmaanbhav.rule.domain.ruleEngine.model.rule.Rule {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()

    // implements rule interface for checking acyclic graph
    override fun getId(): String = id
    override fun ruleType(): String = type
    override fun getInputAttributePaths(): Set<String> = inputAttributes.map { it.id.path }.toSet()
    override fun getOutputAttributePaths(): Set<String> = outputAttributes.map { it.id.path }.toSet()
    override fun getTags(): Set<String> = emptySet()
    override fun getExpression(): String = compiledExpression
}
