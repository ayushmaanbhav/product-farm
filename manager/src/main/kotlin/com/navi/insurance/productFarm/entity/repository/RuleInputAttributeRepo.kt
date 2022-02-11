package com.navi.insurance.productFarm.entity.repository

import com.navi.insurance.productFarm.entity.id.RuleInputAttributeId
import com.navi.insurance.productFarm.entity.relationship.RuleInputAttribute
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface RuleInputAttributeRepo : JpaRepository<RuleInputAttribute, RuleInputAttributeId>
