package io.github.ayushmaanbhav.common.model.response

import org.springframework.http.HttpStatus
import org.springframework.http.ResponseEntity

data class GenericResponse<T>(
    val statusCode: Int? = null, val data: T? = null, val message: String? = null, val errors: Collection<ErrorDetail>? = null
) {
    companion object {
        fun <T> getResponseMessageWithCode(message: String, status: HttpStatus): ResponseEntity<GenericResponse<T>> =
            ResponseEntity.ok(GenericResponse(statusCode = status.value(), message = message))

        fun <T> getResponseWithCode(data: T, status: HttpStatus): ResponseEntity<GenericResponse<T>> =
            ResponseEntity.ok(GenericResponse(statusCode = status.value(), data = data))
    }
}
