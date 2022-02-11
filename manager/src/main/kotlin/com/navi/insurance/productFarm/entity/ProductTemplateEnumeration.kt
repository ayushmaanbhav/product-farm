package com.navi.insurance.productFarm.entity

import com.navi.insurance.productFarm.constant.ProductTemplateType
import com.vladmihalcea.hibernate.type.json.JsonBinaryType
import org.hibernate.annotations.NaturalId
import org.hibernate.annotations.Type
import org.hibernate.annotations.TypeDef
import javax.persistence.Column
import javax.persistence.Entity
import javax.persistence.EnumType
import javax.persistence.Enumerated
import javax.persistence.Id
import javax.persistence.Index
import javax.persistence.Table

@Entity
@Table(
    name = "product_template_enumeration",
    indexes = [
        Index(columnList = "productTemplateType,name", unique = true),
    ]
)
@TypeDef(name = "jsonb", typeClass = JsonBinaryType::class)
data class ProductTemplateEnumeration(
    @Id @NaturalId val id: String,
    val name: String,
    @Enumerated(EnumType.STRING)
    val productTemplateType: ProductTemplateType,
    @Type(type = "jsonb")
    @Column(columnDefinition = "jsonb")
    val value: Set<String>,
    val description: String?,
) : AbstractEntity<ProductTemplateEnumeration>(ProductTemplateEnumeration::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
