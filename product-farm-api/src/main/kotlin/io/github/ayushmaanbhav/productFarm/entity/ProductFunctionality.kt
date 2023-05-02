package io.github.ayushmaanbhav.productFarm.entity

import ValidProductFunctionality
import io.github.ayushmaanbhav.productFarm.constant.ProductFunctionalityStatus
import io.github.ayushmaanbhav.productFarm.entity.relationship.FunctionalityRequiredAttribute
import jakarta.persistence.CascadeType
import jakarta.persistence.Entity
import jakarta.persistence.FetchType
import jakarta.persistence.Id
import jakarta.persistence.Index
import jakarta.persistence.JoinColumn
import jakarta.persistence.OneToMany
import jakarta.persistence.OrderBy
import jakarta.persistence.Table
import org.hibernate.annotations.NaturalId

@Entity
@Table(
    name = "product_functionality",
    indexes = [
        Index(columnList = "productId,name", unique = true),
    ],
)
@ValidProductFunctionality
data class ProductFunctionality(
    @Id @NaturalId val id: String,
    val name: String,
    val productId: String,
    val immutable: Boolean,
    val description: String,
    @OneToMany(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.LAZY)
    @JoinColumn(name = "functionalityId", referencedColumnName = "id")
    @OrderBy("`order`")
    val requiredAttributes: List<FunctionalityRequiredAttribute>,
    var status: ProductFunctionalityStatus,
) : AbstractEntity<ProductFunctionality>(ProductFunctionality::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
