import io.github.ayushmaanbhav.productFarm.entity.validation.AbstractAttributeValidator
import jakarta.validation.Constraint
import jakarta.validation.Payload
import kotlin.annotation.AnnotationRetention.RUNTIME
import kotlin.annotation.AnnotationTarget.ANNOTATION_CLASS
import kotlin.annotation.AnnotationTarget.CLASS
import kotlin.annotation.AnnotationTarget.TYPE
import kotlin.reflect.KClass

@Target(TYPE, ANNOTATION_CLASS, CLASS)
@Retention(RUNTIME)
@MustBeDocumented
@Constraint(validatedBy = [AbstractAttributeValidator::class])
annotation class ValidAbstractAttribute(
    val message: String = "",
    val groups: Array<KClass<*>> = [],
    val payload: Array<KClass<out Payload>> = [],
)
