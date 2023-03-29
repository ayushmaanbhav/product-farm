package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.datatype.dto.DatatypeDto
import io.github.ayushmaanbhav.productFarm.entity.Datatype
import org.springframework.stereotype.Component

@Component
class DatatypeTransformer : Transformer<Datatype, io.github.ayushmaanbhav.productFarm.api.datatype.dto.DatatypeDto>() {
    
    override fun forward(input: Datatype): io.github.ayushmaanbhav.productFarm.api.datatype.dto.DatatypeDto =
            io.github.ayushmaanbhav.productFarm.api.datatype.dto.DatatypeDto(name = input.name, type = input.type, description = input.description)
    
    override fun reverse(input: io.github.ayushmaanbhav.productFarm.api.datatype.dto.DatatypeDto): Datatype =
        Datatype(name = input.name, type = input.type, description = input.description)
}
