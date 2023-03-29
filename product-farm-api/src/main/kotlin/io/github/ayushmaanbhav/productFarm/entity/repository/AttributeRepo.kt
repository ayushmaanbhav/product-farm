package io.github.ayushmaanbhav.productFarm.entity.repository

import io.github.ayushmaanbhav.productFarm.entity.Attribute
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface AttributeRepo : JpaRepository<Attribute, String> {
    fun findAllByProductIdOrderByPathAsc(productId: String): List<Attribute>
    
    fun findAllByAbstractAttribute_AbstractPath(abstractPath: String): List<Attribute>
}
