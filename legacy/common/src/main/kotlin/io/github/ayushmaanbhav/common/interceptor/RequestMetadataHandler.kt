package io.github.ayushmaanbhav.common.interceptor

import io.github.ayushmaanbhav.common.model.RequestMetadata
import io.github.ayushmaanbhav.common.model.RequestMetadata.Companion.APP_VERSION_CODE
import io.github.ayushmaanbhav.common.model.RequestMetadata.Companion.CORRELATION_ID_HEADER
import io.github.ayushmaanbhav.common.model.RequestMetadata.Companion.CUSTOMER_ID_HEADER
import io.github.ayushmaanbhav.common.model.RequestMetadata.Companion.DEVICE_ID
import io.github.ayushmaanbhav.common.model.RequestMetadata.Companion.OS_VERSION
import io.github.ayushmaanbhav.common.model.RequestMetadata.Companion.X_CLICK_STREAM_DATA
import jakarta.servlet.http.HttpServletRequest
import jakarta.servlet.http.HttpServletResponse
import java.util.*
import org.springframework.web.servlet.HandlerInterceptor
import org.springframework.web.servlet.ModelAndView

class RequestMetadataHandler : HandlerInterceptor {
    override fun preHandle(request: HttpServletRequest, response: HttpServletResponse, handler: Any): Boolean {
        (request.getHeader(CORRELATION_ID_HEADER) ?: UUID.randomUUID().toString()).let { RequestMetadata.setCorrelationId(it) }
        request.getHeader(OS_VERSION)?.let { RequestMetadata.setOsVersion(it) }
        request.getHeader(DEVICE_ID)?.let { RequestMetadata.setDeviceId(it) }
        request.getHeader(CUSTOMER_ID_HEADER)?.let { RequestMetadata.setCustomerId(it) }
        request.getHeader(APP_VERSION_CODE)?.let { RequestMetadata.setAppVersionCode(it) }
        request.getHeader(X_CLICK_STREAM_DATA)?.let { RequestMetadata.setClickStreamData(it) }
        return true
    }

    override fun postHandle(request: HttpServletRequest, response: HttpServletResponse, handler: Any, modelAndView: ModelAndView?) {
        RequestMetadata.resetCorrelationId()
        RequestMetadata.resetDeviceId()
        RequestMetadata.resetOsVersion()
        RequestMetadata.resetCustomerId()
        RequestMetadata.resetAppVersionCode()
        RequestMetadata.resetClickStreamData()
    }
}
