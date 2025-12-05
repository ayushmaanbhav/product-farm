package io.github.ayushmaanbhav.common.interceptor

import io.github.ayushmaanbhav.common.model.RequestMetadata
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import io.mockk.every
import io.mockk.mockk
import jakarta.servlet.http.HttpServletRequest
import jakarta.servlet.http.HttpServletResponse
import org.springframework.web.servlet.HandlerInterceptor
import org.springframework.web.servlet.ModelAndView

class RequestMetadataHandlerTest : StringSpec() {
    private val handlerInterceptor: HandlerInterceptor = RequestMetadataHandler()

    init {
        "preHandle should set RequestMetadata variables when headers are present" {
            // Arrange
            val request = mockk<HttpServletRequest>(relaxed = true)
            every { request.getHeader(RequestMetadata.CORRELATION_ID_HEADER) } returns "12345"
            every { request.getHeader(RequestMetadata.OS_VERSION) } returns "Android 10"
            every { request.getHeader(RequestMetadata.DEVICE_ID) } returns "device123"
            every { request.getHeader(RequestMetadata.CUSTOMER_ID_HEADER) } returns "customer123"
            every { request.getHeader(RequestMetadata.APP_VERSION_CODE) } returns "1.0.0"
            every { request.getHeader(RequestMetadata.X_CLICK_STREAM_DATA) } returns "data123"

            val response = mockk<HttpServletResponse>()
            val handler = mockk<Any>()

            // Act
            val result = handlerInterceptor.preHandle(request, response, handler)

            // Assert
            result shouldBe true
            RequestMetadata.getCorrelationId() shouldBe "12345"
            RequestMetadata.getOsVersion() shouldBe "Android 10"
            RequestMetadata.getDeviceId() shouldBe "device123"
            RequestMetadata.getCustomerId() shouldBe "customer123"
            RequestMetadata.getAppVersionCode() shouldBe "1.0.0"
            RequestMetadata.getClickStreamData() shouldBe "data123"
        }

        "preHandle should set correlationId to a random UUID when correlationId header is not present" {
            // Arrange
            val request = mockk<HttpServletRequest>(relaxed = true)
            val response = mockk<HttpServletResponse>()
            val handler = mockk<Any>()

            // Act
            val result = handlerInterceptor.preHandle(request, response, handler)

            // Assert
            result shouldBe true
            RequestMetadata.getCorrelationId() shouldNotBe null
        }

        "postHandle should reset RequestMetadata variables" {
            // Arrange
            RequestMetadata.setCorrelationId("12345")
            RequestMetadata.setOsVersion("Android 10")
            RequestMetadata.setDeviceId("device123")
            RequestMetadata.setCustomerId("customer123")
            RequestMetadata.setAppVersionCode("1.0.0")
            RequestMetadata.setClickStreamData("data123")

            val request = mockk<HttpServletRequest>()
            val response = mockk<HttpServletResponse>()
            val handler = mockk<Any>()
            val modelAndView = mockk<ModelAndView>()

            // Act
            handlerInterceptor.postHandle(request, response, handler, modelAndView)

            // Assert
            RequestMetadata.getCorrelationId() shouldBe null
            RequestMetadata.getOsVersion() shouldBe null
            RequestMetadata.getDeviceId() shouldBe null
            RequestMetadata.getCustomerId() shouldBe null
            RequestMetadata.getAppVersionCode() shouldBe null
            RequestMetadata.getClickStreamData() shouldBe null
        }
    }
}
