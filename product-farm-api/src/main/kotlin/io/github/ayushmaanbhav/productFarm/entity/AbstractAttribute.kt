package io.github.ayushmaanbhav.productFarm.entity

import ValidAbstractAttribute
import io.github.ayushmaanbhav.productFarm.entity.relationship.AbstractAttributeRelatedAttribute
import io.github.ayushmaanbhav.productFarm.entity.relationship.AbstractAttributeTag
import io.github.ayushmaanbhav.productFarm.entity.relationship.AttributeDisplayName
import jakarta.persistence.CascadeType
import jakarta.persistence.Entity
import jakarta.persistence.FetchType
import jakarta.persistence.Id
import jakarta.persistence.Index
import jakarta.persistence.JoinColumn
import jakarta.persistence.OneToMany
import jakarta.persistence.OneToOne
import jakarta.persistence.OrderBy
import jakarta.persistence.Table
import org.hibernate.annotations.NaturalId

@Entity
@Table(
    name = "abstract_attribute",
    indexes = [
        Index(columnList = "productId,componentType,componentId"),
    ],
)
@ValidAbstractAttribute
data class AbstractAttribute(
    @Id @NaturalId val abstractPath: String,
    @OneToMany(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.LAZY)
    @JoinColumn(name = "abstractPath", referencedColumnName = "abstractPath")
    val displayNames: List<AttributeDisplayName>,
    val componentType: String,
    val componentId: String?,
    @OneToMany(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.LAZY)
    @JoinColumn(name = "abstractPath", referencedColumnName = "abstractPath")
    @OrderBy("`order`")
    val tags: List<AbstractAttributeTag>,
    @OneToOne(fetch = FetchType.EAGER)
    @JoinColumn(name = "datatype", referencedColumnName = "name")
    val datatype: Datatype,
    @OneToOne(fetch = FetchType.EAGER)
    @JoinColumn(name = "enumerationId", referencedColumnName = "id")
    val enumeration: ProductTemplateEnumeration?,
    @OneToMany(fetch = FetchType.EAGER)
    @JoinColumn(name = "abstractPath", referencedColumnName = "abstractPath")
    @OrderBy("`order`")
    val relatedAttributes: List<AbstractAttributeRelatedAttribute>,
    @OneToOne(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.LAZY)
    @JoinColumn(name = "constraintRuleId", referencedColumnName = "id")
    val constraintRule: Rule?,
    var immutable: Boolean,
    val description: String?,
    val productId: String,
) : AbstractEntity<AbstractAttribute>(AbstractAttribute::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
