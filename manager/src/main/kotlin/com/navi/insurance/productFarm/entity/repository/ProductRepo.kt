package com.navi.insurance.productFarm.entity.repository

import com.navi.insurance.productFarm.entity.Product
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface ProductRepo : JpaRepository<Product, String>
