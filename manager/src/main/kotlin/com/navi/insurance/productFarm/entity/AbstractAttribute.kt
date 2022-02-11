package com.navi.insurance.productFarm.entity

import com.navi.insurance.productFarm.entity.relationship.AbstractAttributeReferenceAttribute
import com.navi.insurance.productFarm.entity.relationship.AbstractAttributeTag
import com.navi.insurance.productFarm.entity.relationship.AttributeDisplayName
import org.hibernate.annotations.NaturalId
import javax.persistence.CascadeType
import javax.persistence.Entity
import javax.persistence.FetchType
import javax.persistence.Id
import javax.persistence.Index
import javax.persistence.JoinColumn
import javax.persistence.OneToMany
import javax.persistence.OneToOne
import javax.persistence.Table

@Entity
@Table(
    name = "abstract_attribute",
    indexes = [
        Index(columnList = "productId,componentType,componentId"),
    ]
)
data class AbstractAttribute(
    @Id @NaturalId val abstractPath: String,
    @OneToMany(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.EAGER)
    @JoinColumn(name = "abstractPath", referencedColumnName = "abstractPath")
    val displayName: Set<AttributeDisplayName>,
    val componentType: String,
    val componentId: String?,
    @OneToMany(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.EAGER)
    @JoinColumn(name = "abstractPath", referencedColumnName = "abstractPath")
    val tag: Set<AbstractAttributeTag>,
    @OneToOne(fetch = FetchType.EAGER)
    @JoinColumn(name = "datatype", referencedColumnName = "name")
    val datatype: Datatype,
    @OneToOne(fetch = FetchType.EAGER)
    @JoinColumn(name = "enumerationId", referencedColumnName = "id")
    val enumeration: ProductTemplateEnumeration?,
    @OneToMany(fetch = FetchType.EAGER)
    @JoinColumn(name = "abstractPath", referencedColumnName = "abstractPath")
    val referenceAttribute: Set<AbstractAttributeReferenceAttribute>,
    val constraintExpression: String?,
    val immutable: Boolean,
    val description: String?,
    val productId: String,
) : AbstractEntity<AbstractAttribute>(AbstractAttribute::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
