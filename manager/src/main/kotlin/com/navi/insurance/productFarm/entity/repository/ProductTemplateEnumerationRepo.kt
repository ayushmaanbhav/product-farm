package com.navi.insurance.productFarm.entity.repository

import com.navi.insurance.productFarm.constant.ProductTemplateType
import com.navi.insurance.productFarm.entity.ProductTemplateEnumeration
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
}
