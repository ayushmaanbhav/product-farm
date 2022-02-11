package com.navi.insurance.productFarm.entity

import com.navi.insurance.productFarm.constant.ProductStatus
import com.navi.insurance.productFarm.constant.ProductTemplateType
import org.hibernate.annotations.NaturalId
import java.time.LocalDateTime
import javax.persistence.Entity
import javax.persistence.EnumType
import javax.persistence.Enumerated
import javax.persistence.Id
import javax.persistence.Table

@Entity
@Table(name = "product")
data class Product(
    @Id @NaturalId val id: String,
    val name: String,
    @Enumerated(EnumType.STRING)
    val status: ProductStatus,
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
