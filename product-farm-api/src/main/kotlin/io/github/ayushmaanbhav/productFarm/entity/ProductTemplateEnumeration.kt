package io.github.ayushmaanbhav.productFarm.entity

import ValidProductTemplateEnumeration
import io.github.ayushmaanbhav.productFarm.constant.ProductTemplateType
import com.vladmihalcea.hibernate.type.json.JsonBinaryType
import org.hibernate.annotations.NaturalId
import org.hibernate.annotations.Type
import org.hibernate.annotations.TypeDef
import jakarta.persistence.Column
import jakarta.persistence.Entity
import jakarta.persistence.EnumType
import jakarta.persistence.Enumerated
import jakarta.persistence.Id
import jakarta.persistence.Index
import jakarta.persistence.Table

@Entity
@Table(
    name = "product_template_enumeration",
    indexes = [
        Index(columnList = "productTemplateType,name", unique = true),
    ],
)
@TypeDef(name = "jsonb", typeClass = JsonBinaryType::class)
@ValidProductTemplateEnumeration
data class ProductTemplateEnumeration(
    @Id @NaturalId val id: String,
    val name: String,
    @Enumerated(EnumType.STRING)
    val productTemplateType: ProductTemplateType,
    @Type(type = "jsonb")
    @Column(columnDefinition = "jsonb")
    val values: LinkedHashSet<String>,
    val description: String?,
) : AbstractEntity<ProductTemplateEnumeration>(ProductTemplateEnumeration::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
