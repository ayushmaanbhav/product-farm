package io.github.ayushmaanbhav.productFarm.api.datatype

import com.github.lkqm.spring.api.version.ApiVersion
import io.github.ayushmaanbhav.common.model.response.GenericResponse
import io.github.ayushmaanbhav.productFarm.api.datatype.dto.DatatypeDto
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.service.DatatypeService
import org.springframework.http.HttpStatus
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.RestController

@ApiVersion("0")
@RestController
class DatatypeController(
    private val datatypeService: DatatypeService,
) : DatatypeApi {
    override fun create(datatypeDto: DatatypeDto): ResponseEntity<GenericResponse<Nothing>> {
        datatypeService.create(datatypeDto)
        return GenericResponse.getResponseMessageWithCode(Constant.CREATED_MESSAGE, HttpStatus.CREATED)
    }
    
    override fun get(name: String): ResponseEntity<GenericResponse<DatatypeDto?>> =
        datatypeService.get(name).map {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }.orElseGet {
            GenericResponse.getResponseMessageWithCode(Constant.NOT_FOUND_MESSAGE, HttpStatus.NOT_FOUND)
        }
}
