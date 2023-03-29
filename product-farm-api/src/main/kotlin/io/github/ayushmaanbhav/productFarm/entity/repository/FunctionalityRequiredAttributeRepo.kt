package io.github.ayushmaanbhav.productFarm.entity.repository

import io.github.ayushmaanbhav.productFarm.entity.compositeId.FunctionalityRequiredAttributeId
import io.github.ayushmaanbhav.productFarm.entity.relationship.FunctionalityRequiredAttribute
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface FunctionalityRequiredAttributeRepo :
    JpaRepository<FunctionalityRequiredAttribute, FunctionalityRequiredAttributeId>
