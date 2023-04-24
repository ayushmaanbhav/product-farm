package io.github.ayushmaanbhav.jsonLogic.utils.type

open class LinkList<T>(private var head: Node<T>? = null, private var tail: Node<T>? = null): Iterable<T> {
    fun addLast(value: T) {
        val node = Node(value, null, tail)
        if (head == null) head = node
        tail?.next = node
        tail = node
    }

    fun getLastNode(): Node<T> {
        if (tail == null) {
            throw NoSuchElementException("list is empty")
        }
        return tail!!
    }

    fun subList(head: Node<T>, tail: Node<T>): LinkList<T> = SubList(head, tail, this)

    open fun clear() {
        if (head == null || tail == null) return
        if (head!!.prev != null) head!!.prev!!.next = tail!!.next
        if (tail!!.next != null) tail!!.next!!.prev = head!!.prev
        head!!.prev = null
        tail!!.next = null
        head = null
        tail = null
    }

    override fun iterator(): Iterator<T> {
        return object : Iterator<T> {
            var current: Node<T>? = null
            var next: Node<T>? = head
            override fun hasNext(): Boolean = next != null
            override fun next(): T {
                current = next
                next = next!!.next
                return current!!.data
            }
        }
    }

    fun hasSingleElement(): Boolean = head === tail

    class Node<T>(val data: T, var next: Node<T>?, var prev: Node<T>?)

    class SubList<T>(private val head: Node<T>, private val tail: Node<T>, private val parent: LinkList<T>): LinkList<T>(head, tail) {
        override fun clear() {
            if (head == parent.head) {
                parent.head = tail.next
            }
            if (tail == parent.tail) {
                parent.tail = head.prev
            }
            super.clear()
        }
    }
}
