package io.github.ayushmaanbhav.productFarm.entity.relationship

import io.github.ayushmaanbhav.productFarm.constant.DisplayNameFormat
import io.github.ayushmaanbhav.productFarm.entity.api.AbstractEntity
import io.github.ayushmaanbhav.productFarm.entity.compositeId.AttributeDisplayNameId
import jakarta.persistence.Column
import jakarta.persistence.EmbeddedId
import jakarta.persistence.Entity
import jakarta.persistence.EnumType
import jakarta.persistence.Enumerated
import jakarta.persistence.Index
import jakarta.persistence.Table

@Entity
@Table(
    name = "product_display_name",
    indexes = [
        Index(columnList = "abstractPath,displayName", unique = true)
    ],
)
data class AttributeDisplayName(
    @EmbeddedId val id: AttributeDisplayNameId,
    val abstractPath: String?,
    val path: String?,
    @Enumerated(EnumType.STRING)
    val displayNameFormat: DisplayNameFormat,
    @Column(name = "`order`")
    val order: Int,
) : AbstractEntity<AttributeDisplayName>(AttributeDisplayName::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
