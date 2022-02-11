package com.navi.insurance.productFarm.entity.repository

import com.navi.insurance.productFarm.entity.Datatype
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface DatatypeRepo : JpaRepository<Datatype, String>
