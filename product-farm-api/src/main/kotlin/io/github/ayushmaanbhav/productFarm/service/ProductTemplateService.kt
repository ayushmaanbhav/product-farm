package io.github.ayushmaanbhav.productFarm.service

import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.api.productTemplate.dto.ProductTemplateEnumerationDto
import io.github.ayushmaanbhav.productFarm.constant.ProductTemplateType
import io.github.ayushmaanbhav.productFarm.entity.repository.ProductTemplateEnumerationRepo
import io.github.ayushmaanbhav.productFarm.transformer.ProductTemplateEnumerationTransformer
import io.github.ayushmaanbhav.productFarm.util.createError
import jakarta.transaction.Transactional
import java.util.*
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Component

@Component
class ProductTemplateService(
    private val enumerationTransformer: ProductTemplateEnumerationTransformer,
    private val enumerationRepo: ProductTemplateEnumerationRepo,
) {
    @Transactional
    fun createEnumeration(templateType: ProductTemplateType, enumerationDto: ProductTemplateEnumerationDto) {
        if (enumerationRepo.existsByProductTemplateTypeAndName(templateType, enumerationDto.name)) {
            throw ValidatorException(
                HttpStatus.BAD_REQUEST.value(), listOf(createError("Enumeration already exists for this name"))
            )
        }
        enumerationRepo.save(enumerationTransformer.reverse(Pair(enumerationDto, templateType)))
    }
    
    fun getEnumeration(templateType: ProductTemplateType, name: String): Optional<ProductTemplateEnumerationDto> {
        return enumerationRepo.findByProductTemplateTypeAndName(templateType, name).map {
            enumerationTransformer.forward(it).first
        }
    }
}
