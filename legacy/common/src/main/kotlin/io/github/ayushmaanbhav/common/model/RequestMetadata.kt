package io.github.ayushmaanbhav.common.model

import org.apache.logging.log4j.ThreadContext

class RequestMetadata {
    companion object {
        const val CORRELATION_ID_HEADER = "X-Correlation-Id"
        const val CORRELATION_ID = "correlationId"
        const val DEVICE_ID = "deviceId"
        const val OS_VERSION = "osVersion"
        const val CUSTOMER_ID_HEADER = "X-Customer-Id"
        const val CUSTOMER_ID = "customerId"
        const val X_CLICK_STREAM_DATA = "X-Click-Stream-Data"
        const val APP_VERSION_CODE = "appVersionCode"

        fun getCorrelationId(): String? = ThreadContext.get(CORRELATION_ID)
        fun getDeviceId(): String? = ThreadContext.get(DEVICE_ID)
        fun getOsVersion(): String? = ThreadContext.get(OS_VERSION)
        fun getCustomerId(): String? = ThreadContext.get(CUSTOMER_ID)
        fun getClickStreamData(): String? = ThreadContext.get(X_CLICK_STREAM_DATA)
        fun getAppVersionCode(): String? = ThreadContext.get(APP_VERSION_CODE)

        fun setCorrelationId(requestId: String) = ThreadContext.put(CORRELATION_ID, requestId)
        fun setDeviceId(deviceId: String) = ThreadContext.put(DEVICE_ID, deviceId)
        fun setOsVersion(osVersion: String) = ThreadContext.put(OS_VERSION, osVersion)
        fun setCustomerId(customerId: String) = ThreadContext.put(CUSTOMER_ID, customerId)
        fun setClickStreamData(clickStreamData: String) = ThreadContext.put(X_CLICK_STREAM_DATA, clickStreamData)
        fun setAppVersionCode(appVersionCode: String) = ThreadContext.put(APP_VERSION_CODE, appVersionCode)

        fun resetCorrelationId() = ThreadContext.remove(CORRELATION_ID)
        fun resetDeviceId() = ThreadContext.remove(DEVICE_ID)
        fun resetOsVersion() = ThreadContext.remove(OS_VERSION)
        fun resetCustomerId() = ThreadContext.remove(CUSTOMER_ID)
        fun resetClickStreamData() = ThreadContext.remove(X_CLICK_STREAM_DATA)
        fun resetAppVersionCode() = ThreadContext.remove(APP_VERSION_CODE)
    }
}
