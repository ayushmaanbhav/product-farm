package io.github.ayushmaanbhav.productFarm.service

import io.github.ayushmaanbhav.common.validator.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.api.datatype.dto.DatatypeDto
import io.github.ayushmaanbhav.productFarm.entity.repository.DatatypeRepo
import io.github.ayushmaanbhav.productFarm.transformer.DatatypeTransformer
import io.github.ayushmaanbhav.productFarm.validation.createError
import org.springframework.http.HttpStatus.BAD_REQUEST
import org.springframework.stereotype.Component
import java.util.*
import jakarta.transaction.Transactional

@Component
class DatatypeService(
    private val datatypeTransformer: DatatypeTransformer,
    private val datatypeRepo: DatatypeRepo,
) {
    @Transactional
    fun create(datatypeDto: DatatypeDto) {
        if (datatypeRepo.existsById(datatypeDto.name)) {
            throw ValidatorException(
                BAD_REQUEST.value(), listOf(createError("Datatype already exists for this name"))
            )
        }
        datatypeRepo.save(datatypeTransformer.reverse(datatypeDto))
    }
    
    fun get(name: String): Optional<DatatypeDto> =
        datatypeRepo.findById(name).map { datatypeTransformer.forward(it) }
}
