package com.navi.insurance.productFarm.entity.repository

import com.navi.insurance.productFarm.entity.id.RuleOutputAttributeId
import com.navi.insurance.productFarm.entity.relationship.RuleOutputAttribute
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface RuleOutputAttributeRepo : JpaRepository<RuleOutputAttribute, RuleOutputAttributeId>
