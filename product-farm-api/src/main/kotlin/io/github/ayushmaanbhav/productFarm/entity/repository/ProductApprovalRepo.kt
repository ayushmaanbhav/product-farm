package io.github.ayushmaanbhav.productFarm.entity.repository

import io.github.ayushmaanbhav.productFarm.entity.ProductApproval
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface ProductApprovalRepo : JpaRepository<ProductApproval, String>
