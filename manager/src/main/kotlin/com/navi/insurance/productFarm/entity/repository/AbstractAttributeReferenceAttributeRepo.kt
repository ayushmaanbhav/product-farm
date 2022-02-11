package com.navi.insurance.productFarm.entity.repository

import com.navi.insurance.productFarm.entity.id.AbstractAttributeReferenceAttributeId
import com.navi.insurance.productFarm.entity.relationship.AbstractAttributeReferenceAttribute
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface AbstractAttributeReferenceAttributeRepo :
    JpaRepository<AbstractAttributeReferenceAttribute, AbstractAttributeReferenceAttributeId>
