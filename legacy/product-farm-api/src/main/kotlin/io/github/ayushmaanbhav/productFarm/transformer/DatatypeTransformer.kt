package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.datatype.dto.DatatypeDto
import io.github.ayushmaanbhav.productFarm.entity.Datatype
import org.springframework.stereotype.Component

@Component
class DatatypeTransformer : Transformer<Datatype, DatatypeDto> {
    
    override fun forward(input: Datatype): DatatypeDto =
        DatatypeDto(name = input.name, type = input.type, description = input.description)
    
    override fun reverse(input: DatatypeDto): Datatype =
        Datatype(name = input.name, type = input.type, description = input.description)
}
