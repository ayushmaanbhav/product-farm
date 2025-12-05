package io.github.ayushmaanbhav.jsonLogic.config

data class StreamProcessingConfig(val maxOperatorStackLimitWithoutReduction: Int, val operatorsIneligibleForReduction: Set<String>) {
    companion object {
        internal const val DEFAULT_MAX_OPERATOR_STACK_LIMIT_WITHOUT_REDUCTION = 100 // to prevent stack overflow as evaluation process is recursive
        internal val DEFAULT_OPERATORS_INELIGIBLE_FOR_REDUCTION = setOf("if") // to optimise on the specific branches that'll need reduction

        val DEFAULT = StreamProcessingConfig(DEFAULT_MAX_OPERATOR_STACK_LIMIT_WITHOUT_REDUCTION, DEFAULT_OPERATORS_INELIGIBLE_FOR_REDUCTION)
    }
}
