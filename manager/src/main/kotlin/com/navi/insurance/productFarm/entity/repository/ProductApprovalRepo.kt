package com.navi.insurance.productFarm.entity.repository

import com.navi.insurance.productFarm.entity.ProductApproval
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface ProductApprovalRepo : JpaRepository<ProductApproval, String>
