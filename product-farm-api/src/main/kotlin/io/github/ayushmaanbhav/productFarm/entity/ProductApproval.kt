package io.github.ayushmaanbhav.productFarm.entity

import ValidProductApproval
import jakarta.persistence.Entity
import jakarta.persistence.Id
import jakarta.persistence.Table
import org.hibernate.annotations.NaturalId

@Entity
@Table(name = "product_approval")
@ValidProductApproval
data class ProductApproval(
    @Id @NaturalId val productId: String,
    val approvedBy: String,
    val discontinuedProductId: String?,
    val changeDescription: String,
) : AbstractEntity<ProductApproval>(ProductApproval::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
