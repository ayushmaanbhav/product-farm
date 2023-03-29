package io.github.ayushmaanbhav.productFarm.entity.repository

import io.github.ayushmaanbhav.productFarm.entity.compositeId.RuleInputAttributeId
import io.github.ayushmaanbhav.productFarm.entity.relationship.RuleInputAttribute
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface RuleInputAttributeRepo : JpaRepository<RuleInputAttribute, RuleInputAttributeId>
