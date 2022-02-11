package com.navi.insurance.productFarm.entity.repository

import com.navi.insurance.productFarm.entity.id.FunctionalityRequiredAttributeId
import com.navi.insurance.productFarm.entity.relationship.FunctionalityRequiredAttribute
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface FunctionalityRequiredAttributeRepo :
    JpaRepository<FunctionalityRequiredAttribute, FunctionalityRequiredAttributeId>
