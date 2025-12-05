package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.product.dto.ProductApprovalRequest
import io.github.ayushmaanbhav.productFarm.entity.ProductApproval
import org.springframework.stereotype.Component

@Component
class CreateProductApprovalTransformer : OneWayTransformer<Pair<String, ProductApprovalRequest>, ProductApproval> {

    override fun forward(input: Pair<String, ProductApprovalRequest>) =
        ProductApproval(
            productId = input.first,
            approvedBy = input.second.approvedBy,
            discontinuedProductId = input.second.discontinuedProductId,
            changeDescription = input.second.changeDescription,
        )
}
