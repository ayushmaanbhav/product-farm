package com.navi.insurance.productFarm.entity.repository

import com.navi.insurance.productFarm.entity.id.AttributeDisplayNameId
import com.navi.insurance.productFarm.entity.relationship.AttributeDisplayName
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface AttributeDisplayNameRepo : JpaRepository<AttributeDisplayName, AttributeDisplayNameId>
