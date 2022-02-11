package com.navi.insurance.productFarm.entity.repository

import com.navi.insurance.productFarm.entity.id.AbstractAttributeTagId
import com.navi.insurance.productFarm.entity.relationship.AbstractAttributeTag
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface AbstractAttributeTagRepo : JpaRepository<AbstractAttributeTag, AbstractAttributeTagId>
