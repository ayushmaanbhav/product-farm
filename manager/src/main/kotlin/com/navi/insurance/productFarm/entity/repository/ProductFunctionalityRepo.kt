package com.navi.insurance.productFarm.entity.repository

import com.navi.insurance.productFarm.entity.ProductFunctionality
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface ProductFunctionalityRepo : JpaRepository<ProductFunctionality, String>
