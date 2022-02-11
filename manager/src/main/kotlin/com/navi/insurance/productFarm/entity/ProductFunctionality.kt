package com.navi.insurance.productFarm.entity

import com.navi.insurance.productFarm.entity.relationship.FunctionalityRequiredAttribute
import org.hibernate.annotations.NaturalId
import javax.persistence.CascadeType
import javax.persistence.Entity
import javax.persistence.FetchType
import javax.persistence.Id
import javax.persistence.Index
import javax.persistence.JoinColumn
import javax.persistence.OneToMany
import javax.persistence.Table

@Entity
@Table(
    name = "product_functionality",
    indexes = [
        Index(columnList = "productId,name", unique = true),
    ]
)
data class ProductFunctionality(
    @Id @NaturalId val id: String,
    val name: String,
    val productId: String,
    val immutable: Boolean,
    val description: String,
    @OneToMany(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.LAZY)
    @JoinColumn(name = "functionalityId", referencedColumnName = "id")
    val requiredAttribute: Set<FunctionalityRequiredAttribute>
) : AbstractEntity<ProductFunctionality>(ProductFunctionality::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
