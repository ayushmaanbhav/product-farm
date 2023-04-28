package io.github.ayushmaanbhav.common.util

import io.github.ayushmaanbhav.common.model.RequestMetadata
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import java.util.*
import java.util.concurrent.CompletableFuture
import java.util.concurrent.Executors
import org.apache.logging.log4j.ThreadContext

class CompletableFutureWithCtxTest : StringSpec({

    "runAsync should execute the given runnable in a new thread with the ThreadContext variables set and removed correctly" {
        val correlationId = UUID.randomUUID().toString()
        ThreadContext.put(RequestMetadata.CORRELATION_ID, correlationId)
        val executorService = Executors.newCachedThreadPool()
        CompletableFutureWithCtx.runAsync(executorService) {
            correlationId shouldBe ThreadContext.get(RequestMetadata.CORRELATION_ID)
        }.join()
    }

    "supplyAsync should execute the given supplier in a new thread with the ThreadContext variables set and removed correctly" {
        val correlationId = UUID.randomUUID().toString()
        val randomString = UUID.randomUUID().toString()
        ThreadContext.put(RequestMetadata.CORRELATION_ID, correlationId)
        val executorService = Executors.newCachedThreadPool()
        val completableFuture: CompletableFuture<String> = CompletableFutureWithCtx.supplyAsync(executorService)         {
            correlationId shouldBe ThreadContext.get(RequestMetadata.CORRELATION_ID)
            randomString
        }
        randomString shouldBe completableFuture.get()
    }
})
