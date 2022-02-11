package com.navi.insurance.productFarm.service

import com.fasterxml.jackson.databind.ObjectMapper
import com.navi.insurance.productFarm.constant.ProductTemplateType
import com.navi.insurance.productFarm.dto.ProductTemplateEnumerationDto
import com.navi.insurance.productFarm.entity.ProductTemplateEnumeration

import com.navi.insurance.productFarm.entity.repository.ProductTemplateEnumerationRepo
import org.springframework.stereotype.Component
import java.util.*
import javax.transaction.Transactional

@Component
class ProductTemplateService(
    val objectMapper: ObjectMapper,
    val enumerationRepo: ProductTemplateEnumerationRepo,
) {
    @Transactional
    fun createEnumeration(templateType: ProductTemplateType, enumerationDto: ProductTemplateEnumerationDto) {
        if (enumerationRepo.existsByProductTemplateTypeAndName(templateType, enumerationDto.name)) {
            throw UnsupportedOperationException("Enumeration already exists for this name")
        }
        val datatype = objectMapper.convertValue(enumerationDto, ProductTemplateEnumeration::class.java)
        enumerationRepo.save(datatype)
    }
    
    fun getEnumeration(templateType: ProductTemplateType, name: String): Optional<ProductTemplateEnumerationDto> {
        return enumerationRepo.findByProductTemplateTypeAndName(templateType, name).map {
            objectMapper.convertValue(it, ProductTemplateEnumerationDto::class.java)
        }
    }
}
