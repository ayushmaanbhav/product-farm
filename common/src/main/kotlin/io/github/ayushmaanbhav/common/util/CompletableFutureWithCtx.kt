package io.github.ayushmaanbhav.common.util

import io.github.ayushmaanbhav.common.model.RequestMetadata.Companion.CORRELATION_ID
import java.util.concurrent.CompletableFuture
import java.util.concurrent.ExecutorService
import java.util.function.Supplier
import org.apache.logging.log4j.ThreadContext

class CompletableFutureWithCtx {
    companion object {
        fun runAsync(runnable: Runnable): CompletableFuture<Void> {
            val runnableWithThreadContext = runAsyncWithThreadContext(runnable)
            return CompletableFuture.runAsync(runnableWithThreadContext)
        }

        fun runAsync(executorService: ExecutorService, runnable: Runnable): CompletableFuture<Void> {
            val runnableWithThreadContext = runAsyncWithThreadContext(runnable)
            return CompletableFuture.runAsync(runnableWithThreadContext, executorService)
        }

        fun <U> supplyAsync(supplier: Supplier<U>): CompletableFuture<U> {
            val supplierWithThreadContext = supplyAsyncWithThreadContext(supplier)
            return CompletableFuture.supplyAsync(supplierWithThreadContext)
        }

        fun <U> supplyAsync(executorService: ExecutorService, supplier: Supplier<U>): CompletableFuture<U> {
            val supplierWithThreadContext = supplyAsyncWithThreadContext(supplier)
            return CompletableFuture.supplyAsync(supplierWithThreadContext, executorService)
        }

        private fun runAsyncWithThreadContext(runnable: Runnable): Runnable {
            val correlationId = ThreadContext.get(CORRELATION_ID)
            return Runnable {
                ThreadContext.put(CORRELATION_ID, correlationId)
                try {
                    runnable.run()
                } finally {
                    ThreadContext.remove(CORRELATION_ID)
                }
            }
        }

        private fun <U> supplyAsyncWithThreadContext(supplier: Supplier<U>): Supplier<U> {
            val correlationId = ThreadContext.get(CORRELATION_ID)
            return Supplier {
                ThreadContext.put(CORRELATION_ID, correlationId)
                try {
                    return@Supplier supplier.get()
                } finally {
                    ThreadContext.remove(CORRELATION_ID)
                }
            }
        }
    }
}
