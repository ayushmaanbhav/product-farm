package com.navi.insurance.productFarm.service

import com.fasterxml.jackson.databind.ObjectMapper
import com.navi.insurance.productFarm.dto.DatatypeDto
import com.navi.insurance.productFarm.entity.Datatype
import com.navi.insurance.productFarm.entity.repository.DatatypeRepo
import org.springframework.stereotype.Component
import java.util.*
import javax.transaction.Transactional

@Component
class DatatypeService(
    val objectMapper: ObjectMapper,
    val datatypeRepo: DatatypeRepo,
) {
    @Transactional
    fun create(datatypeDto: DatatypeDto) {
        if (datatypeRepo.existsById(datatypeDto.name)) {
            throw UnsupportedOperationException("Datatype already exists for this name")
        }
        val datatype = objectMapper.convertValue(datatypeDto, Datatype::class.java)
        datatypeRepo.save(datatype)
    }
    
    fun get(name: String): Optional<DatatypeDto> =
        datatypeRepo.findById(name).map { objectMapper.convertValue(it, DatatypeDto::class.java) }
}
