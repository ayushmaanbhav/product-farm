package io.github.ayushmaanbhav.productFarm.entity

import org.hibernate.annotations.CreationTimestamp
import org.hibernate.annotations.UpdateTimestamp
import org.springframework.data.util.ProxyUtils
import java.time.LocalDateTime
import java.util.*
import jakarta.persistence.EmbeddedId
import jakarta.persistence.Id
import jakarta.persistence.MappedSuperclass
import jakarta.persistence.Transient
import jakarta.persistence.Version
import kotlin.reflect.KClass
import kotlin.reflect.cast
import kotlin.reflect.full.memberProperties

@MappedSuperclass
abstract class AbstractEntity<T : AbstractEntity<T>>(@Transient val _type: KClass<T>) {
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
                    val property = it.get(_type.cast(this))
                    val otherProperty = it.get(_type.cast(other))
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
        val values = idProperties.map { it.get(_type.cast(this)) }
        return Objects.hash(values)
    }
    
    @Transient
    private val idProperties = _type.memberProperties.filter {
        it.annotations.any { annotation ->
            annotation.annotationClass in setOf(Id::class, EmbeddedId::class)
        }
    }
}
