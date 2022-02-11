package com.navi.insurance.productFarm.entity.repository

import com.navi.insurance.productFarm.entity.Attribute
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface AttributeRepo : JpaRepository<Attribute, String>
