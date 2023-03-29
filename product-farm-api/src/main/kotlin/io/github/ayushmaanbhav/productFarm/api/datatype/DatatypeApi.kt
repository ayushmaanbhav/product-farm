package io.github.ayushmaanbhav.productFarm.api.datatype

import io.github.ayushmaanbhav.common.model.response.GenericResponse
import io.github.ayushmaanbhav.productFarm.api.datatype.dto.DatatypeDto
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PathVariable
import org.springframework.web.bind.annotation.PutMapping
import org.springframework.web.bind.annotation.RequestBody
import org.springframework.web.bind.annotation.RequestMapping

@RequestMapping("/datatype")
interface DatatypeApi {
    @PutMapping
    fun create(@RequestBody datatypeDto: DatatypeDto): ResponseEntity<GenericResponse<Nothing>>
    
    @GetMapping("/{name}")
    fun get(@PathVariable name: String): ResponseEntity<GenericResponse<DatatypeDto?>>
}
