package io.github.ayushmaanbhav.jsonLogic

sealed class JsonLogicResult {
    data class Success(val value: Any) : JsonLogicResult()

    sealed class Failure : JsonLogicResult() {
        object NullResult : Failure()
        object EmptyExpression : Failure()
        object MissingOperation : Failure()
        object InvalidFormat : Failure()
        object StreamIOError : Failure()
    }

}
