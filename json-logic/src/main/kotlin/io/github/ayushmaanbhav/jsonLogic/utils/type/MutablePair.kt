package io.github.ayushmaanbhav.jsonLogic.utils.type

sealed class Pair<K, V>(open val first: K, open val second: V)

data class MutablePair<K, V>(override val first: K, override var second: V): Pair<K, V>(first, second)
