package io.github.ayushmaanbhav.productFarm.entity

import ValidProductTemplateEnumeration
import com.vladmihalcea.hibernate.type.json.JsonType
import io.github.ayushmaanbhav.productFarm.constant.ProductTemplateType
import io.github.ayushmaanbhav.productFarm.entity.api.AbstractEntity
import jakarta.persistence.Column
import jakarta.persistence.Entity
import jakarta.persistence.EnumType
import jakarta.persistence.Enumerated
import jakarta.persistence.Id
import jakarta.persistence.Index
import jakarta.persistence.Table
import org.hibernate.annotations.NaturalId
import org.hibernate.annotations.Type

@Entity
@Table(
    name = "product_template_enumeration",
    indexes = [
        Index(columnList = "productTemplateType,name", unique = true),
    ],
)
@ValidProductTemplateEnumeration
data class ProductTemplateEnumeration(
    @Id @NaturalId val id: String,
    val name: String,
    @Enumerated(EnumType.STRING)
    val productTemplateType: ProductTemplateType,
    @Type(JsonType::class)
    @Column(columnDefinition = "jsonb")
    val values: LinkedHashSet<String>,
    val description: String?,
) : AbstractEntity<ProductTemplateEnumeration>(ProductTemplateEnumeration::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
