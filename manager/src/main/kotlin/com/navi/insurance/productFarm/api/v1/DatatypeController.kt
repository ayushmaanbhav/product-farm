package com.navi.insurance.productFarm.api.v1

import com.navi.insurance.common.model.response.GenericResponse
import com.navi.insurance.productFarm.constant.Constant
import com.navi.insurance.productFarm.dto.DatatypeDto
import com.navi.insurance.productFarm.service.DatatypeService
import org.springframework.http.HttpStatus
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PathVariable
import org.springframework.web.bind.annotation.PutMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.RestController

@RestController
@RequestMapping("/v1/datatype")
class DatatypeController(
    val datatypeService: DatatypeService
) {
    @PutMapping
    fun create(datatypeDto: DatatypeDto): ResponseEntity<GenericResponse<Nothing>> {
        datatypeService.create(datatypeDto)
        return GenericResponse.getResponseMessageWithCode(Constant.CREATED_MESSAGE, HttpStatus.CREATED)
    }
    
    @GetMapping("/{name}")
    fun get(@PathVariable name: String): ResponseEntity<GenericResponse<DatatypeDto?>> =
        datatypeService.get(name).map {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }.orElseGet {
            GenericResponse.getResponseMessageWithCode(Constant.NOT_FOUND_MESSAGE, HttpStatus.NOT_FOUND)
        }
}
