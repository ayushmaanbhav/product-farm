import io.github.ayushmaanbhav.productFarm.entity.validation.ProductApprovalValidator
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
@Constraint(validatedBy = [ProductApprovalValidator::class])
annotation class ValidProductApproval(
    val message: String = "",
    val groups: Array<KClass<*>> = [],
    val payload: Array<KClass<out Payload>> = [],
)
