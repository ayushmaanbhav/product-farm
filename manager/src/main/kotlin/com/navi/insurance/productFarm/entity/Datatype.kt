package com.navi.insurance.productFarm.entity

import com.navi.insurance.productFarm.constant.DatatypeType
import org.hibernate.annotations.NaturalId
import javax.persistence.Entity
import javax.persistence.EnumType
import javax.persistence.Enumerated
import javax.persistence.Id
import javax.persistence.Table

@Entity
@Table(name = "datatype")
data class Datatype(
    @Id @NaturalId val name: String,
    @Enumerated(EnumType.STRING)
    val type: DatatypeType,
    val description: String?,
) : AbstractEntity<Datatype>(Datatype::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
