package com.navi.insurance.productFarm.entity

import com.navi.insurance.productFarm.constant.AttributeValueType
import com.vladmihalcea.hibernate.type.json.JsonBinaryType
import org.hibernate.annotations.NaturalId
import org.hibernate.annotations.Type
import org.hibernate.annotations.TypeDef
import javax.persistence.CascadeType
import javax.persistence.Column
import javax.persistence.Entity
import javax.persistence.EnumType
import javax.persistence.Enumerated
import javax.persistence.FetchType
import javax.persistence.Id
import javax.persistence.JoinColumn
import javax.persistence.OneToOne
import javax.persistence.Table

@Entity
@Table(name = "attribute")
@TypeDef(name = "jsonb", typeClass = JsonBinaryType::class)
data class Attribute(
    @Id @NaturalId val path: String,
    val abstractPath: String,
    @Enumerated(EnumType.STRING)
    val type: AttributeValueType,
    @Type(type = "jsonb")
    @Column(columnDefinition = "jsonb")
    val value: String?,
    @OneToOne(cascade = [CascadeType.ALL], orphanRemoval = true, fetch = FetchType.EAGER)
    @JoinColumn(name = "ruleId", referencedColumnName = "id")
    val rule: Rule?,
    val productId: String,
) : AbstractEntity<Attribute>(Attribute::class) {
    override fun equals(other: Any?) = super.equals(other)
    override fun hashCode() = super.hashCode()
}
