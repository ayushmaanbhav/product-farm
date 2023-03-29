package io.github.ayushmaanbhav.productFarm.entity.repository

import io.github.ayushmaanbhav.productFarm.entity.ProductFunctionality
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface ProductFunctionalityRepo : JpaRepository<ProductFunctionality, String>
