package io.github.ayushmaanbhav.productFarm.entity.repository

import io.github.ayushmaanbhav.productFarm.entity.compositeId.AbstractAttributeTagId
import io.github.ayushmaanbhav.productFarm.entity.relationship.AbstractAttributeTag
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository

@Repository
interface AbstractAttributeTagRepo : JpaRepository<AbstractAttributeTag, AbstractAttributeTagId>  {
    fun getByProductIdAndTag(productId: String, tag: String): List<AbstractAttributeTag>
}
