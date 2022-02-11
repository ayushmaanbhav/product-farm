package com.navi.insurance.productFarm.entity

import org.hibernate.annotations.CreationTimestamp
import org.hibernate.annotations.UpdateTimestamp
import org.springframework.data.util.ProxyUtils
import java.time.LocalDateTime
import java.util.*
import javax.persistence.EmbeddedId
import javax.persistence.Id
import javax.persistence.MappedSuperclass
import javax.persistence.Transient
import javax.persistence.Version
import kotlin.reflect.KClass
import kotlin.reflect.full.memberProperties

@MappedSuperclass
abstract class AbstractEntity<T : AbstractEntity<T>>(type: KClass<T>) {
    @CreationTimestamp
    private lateinit var createdAt: LocalDateTime
    @UpdateTimestamp
    private lateinit var updatedAt: LocalDateTime
    @Version
    private var version: Long = 0
    
    /**
     * equals implementation for jpa, by default
     * data class's equals compares all properties,
     * this one compares only @Id annotated properties
     */
    override fun equals(other: Any?): Boolean {
        return when {
            other == null -> false
            this === other -> true
            this.javaClass != ProxyUtils.getUserClass(other) -> false
            else -> {
                return idProperties.all {
                    val property = it.get(this as T)
                    val otherProperty = it.get(other as T)
                    property == otherProperty
                }
            }
        }
    }
    
    /**
     * hashCode implementation for jpa, by default
     * data class's hashCode computes on all properties,
     * this one computes only on @Id annotated properties
     */
    override fun hashCode(): Int {
        val values = idProperties.map { it.get(this as T) }
        return Objects.hash(values)
    }
    
    @Transient
    private val idProperties = type.memberProperties.filter {
        it.annotations.any { annotation ->
            annotation.annotationClass in setOf(Id::class, EmbeddedId::class)
        }
    }
}
