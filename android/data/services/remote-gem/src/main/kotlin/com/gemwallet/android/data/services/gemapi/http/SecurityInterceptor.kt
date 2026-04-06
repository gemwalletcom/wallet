package com.gemwallet.android.data.services.gemapi.http

import com.gemwallet.android.application.device.coordinators.GetDeviceId
import okhttp3.Interceptor
import okhttp3.Protocol
import okhttp3.Response
import okio.Buffer

class SecurityInterceptor(
    private val getDeviceId: GetDeviceId,
) : Interceptor {

    private val signer = DeviceRequestSigner(getDeviceId)

    override fun intercept(chain: Interceptor.Chain): Response {
        val request = chain.request()
        val body = request.body?.let {
            val buffer = Buffer()
            it.writeTo(buffer)
            buffer.readUtf8().toByteArray()
        }
        val signature = signer.sign(request.method, request.url.encodedPath, body)
        return try {
            val builder = request.newBuilder()
            signature.toHeaders().forEach { (key, value) -> builder.header(key, value) }
            chain.proceed(builder.build())
        } catch (error: Throwable) {
            Response.Builder()
                .code(503)
                .message("HTTP Exception: ${error.message}")
                .request(request)
                .protocol(Protocol.HTTP_2)
                .build()
        }
    }
}
