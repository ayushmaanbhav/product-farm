package io.github.ayushmaanbhav.productFarm.entity.repository

import io.github.ayushmaanbhav.productFarm.constant.ProductTemplateType
import io.github.ayushmaanbhav.productFarm.entity.ProductTemplateEnumeration
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository
import java.util.*

@Repository
interface ProductTemplateEnumerationRepo : JpaRepository<ProductTemplateEnumeration, String> {
    
    fun existsByProductTemplateTypeAndName(productTemplateType: ProductTemplateType, name: String): Boolean
    
    fun findByProductTemplateTypeAndName(
        productTemplateType: ProductTemplateType,
        name: String
    ): Optional<ProductTemplateEnumeration>
    
    fun getByProductTemplateTypeAndName(
        productTemplateType: ProductTemplateType,
        name: String
    ): ProductTemplateEnumeration
}
