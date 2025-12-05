package io.github.ayushmaanbhav.productFarm.entity.repository

import io.github.ayushmaanbhav.productFarm.entity.ProductFunctionality
import java.util.*
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface ProductFunctionalityRepo : JpaRepository<ProductFunctionality, String> {
    fun existsByProductIdAndName(productId: String, name: String): Boolean
    fun findByProductIdAndName(productId: String, name: String): Optional<ProductFunctionality>
}
