package io.github.ayushmaanbhav.productFarm.entity

import ValidProduct
import io.github.ayushmaanbhav.productFarm.constant.ProductStatus
import io.github.ayushmaanbhav.productFarm.constant.ProductTemplateType
import jakarta.persistence.Entity
import jakarta.persistence.EnumType
import jakarta.persistence.Enumerated
import jakarta.persistence.Id
import jakarta.persistence.Table
import java.time.LocalDateTime
import org.hibernate.annotations.NaturalId

@Entity
@Table(name = "product")
@ValidProduct
data class Product(
    @Id @NaturalId val id: String,
    val name: String,
    @Enumerated(EnumType.STRING)
    var status: ProductStatus,
    val effectiveFrom: LocalDateTime,
    val expiryAt: LocalDateTime,
    @Enumerated(EnumType.STRING)
    val templateType: ProductTemplateType,
    val parentProductId: String?,
    val description: String?,
) : AbstractEntity<Product>(Product::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
