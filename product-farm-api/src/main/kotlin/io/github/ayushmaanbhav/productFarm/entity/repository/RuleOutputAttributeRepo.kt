package io.github.ayushmaanbhav.productFarm.entity.repository

import io.github.ayushmaanbhav.productFarm.entity.compositeId.RuleOutputAttributeId
import io.github.ayushmaanbhav.productFarm.entity.relationship.RuleOutputAttribute
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface RuleOutputAttributeRepo : JpaRepository<RuleOutputAttribute, RuleOutputAttributeId>
