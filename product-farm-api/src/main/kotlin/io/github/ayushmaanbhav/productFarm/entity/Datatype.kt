package io.github.ayushmaanbhav.productFarm.entity

import ValidDatatype
import io.github.ayushmaanbhav.productFarm.constant.DatatypeType
import jakarta.persistence.Entity
import jakarta.persistence.EnumType
import jakarta.persistence.Enumerated
import jakarta.persistence.Id
import jakarta.persistence.Table
import org.hibernate.annotations.NaturalId

@Entity
@Table(name = "datatype")
@ValidDatatype
data class Datatype(
    @Id @NaturalId val name: String,
    @Enumerated(EnumType.STRING)
    val type: DatatypeType,
    val description: String?,
) : AbstractEntity<Datatype>(Datatype::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
