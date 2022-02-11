package com.navi.insurance.productFarm.entity

import org.hibernate.annotations.NaturalId
import java.time.LocalDateTime
import javax.persistence.Entity
import javax.persistence.Id
import javax.persistence.Table

@Entity
@Table(name = "product_approval")
data class ProductApproval(
    @Id @NaturalId val productId: String,
    val approvedBy: String,
    val discontinuedProductId: String?,
    val changeDescription: LocalDateTime,
) : AbstractEntity<ProductApproval>(ProductApproval::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
