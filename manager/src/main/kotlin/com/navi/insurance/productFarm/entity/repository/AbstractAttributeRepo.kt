package com.navi.insurance.productFarm.entity.repository

import com.navi.insurance.productFarm.entity.AbstractAttribute
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface AbstractAttributeRepo : JpaRepository<AbstractAttribute, String>
