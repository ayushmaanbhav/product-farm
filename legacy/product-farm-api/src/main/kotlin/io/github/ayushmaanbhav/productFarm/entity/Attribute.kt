package io.github.ayushmaanbhav.productFarm.entity

import ValidAttribute
import ValidAttributeDirectedAcyclicGraph
import com.fasterxml.jackson.databind.JsonNode
import com.vladmihalcea.hibernate.type.json.JsonType
import io.github.ayushmaanbhav.productFarm.constant.AttributeValueType
import io.github.ayushmaanbhav.productFarm.entity.api.AbstractEntity
import io.github.ayushmaanbhav.productFarm.entity.relationship.AttributeDisplayName
import jakarta.persistence.CascadeType
import jakarta.persistence.Column
import jakarta.persistence.Entity
import jakarta.persistence.EnumType
import jakarta.persistence.Enumerated
import jakarta.persistence.FetchType
import jakarta.persistence.Id
import jakarta.persistence.JoinColumn
import jakarta.persistence.ManyToOne
import jakarta.persistence.OneToMany
import jakarta.persistence.OneToOne
import jakarta.persistence.Table
import org.hibernate.annotations.NaturalId
import org.hibernate.annotations.Type

@Entity
@Table(name = "attribute")
@ValidAttribute
@ValidAttributeDirectedAcyclicGraph
data class Attribute(
    @Id @NaturalId val path: String,
    @OneToMany(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.LAZY)
    @JoinColumn(name = "path", referencedColumnName = "path")
    val displayNames: List<AttributeDisplayName>,
    @ManyToOne(fetch = FetchType.EAGER)
    @JoinColumn(name = "abstractPath", referencedColumnName = "abstractPath")
    val abstractAttribute: AbstractAttribute,
    @Enumerated(EnumType.STRING)
    val type: AttributeValueType,
    @Type(JsonType::class)
    @Column(columnDefinition = "jsonb")
    val value: JsonNode?,
    @OneToOne(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.EAGER)
    @JoinColumn(name = "ruleId", referencedColumnName = "id")
    val rule: Rule?,
    val productId: String,
) : AbstractEntity<Attribute>(Attribute::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
