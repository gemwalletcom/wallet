package com.gemwallet.android.data.services.gemapi.http

import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.wallet.core.primitives.WalletId
import okhttp3.Interceptor
import okhttp3.Protocol
import okhttp3.Request
import okhttp3.Response
import okio.Buffer

const val DEVICE_AUTH_ERROR_CODE = 599

class SecurityInterceptor internal constructor(
    private val signer: DeviceRequestSigner,
) : Interceptor {

    constructor(getDeviceId: GetDeviceId) : this(GemDeviceRequestSigner(getDeviceId))

    override fun intercept(chain: Interceptor.Chain): Response {
        val request = chain.request()
        val body = request.body?.let {
            val buffer = Buffer()
            it.writeTo(buffer)
            buffer.readByteArray()
        }
        val signature = try {
            signer.sign(
                method = request.method,
                path = request.url.encodedPath,
                body = body,
                walletId = request.tag(WalletId::class.java)?.id.orEmpty(),
            )
        } catch (error: Throwable) {
            return request.errorResponse(DEVICE_AUTH_ERROR_CODE, "Device auth error: ${error.javaClass.simpleName}")
        }
        return try {
            val builder = request.newBuilder()
            signature.toHeaders().forEach { (key, value) -> builder.header(key, value) }
            chain.proceed(builder.build())
        } catch (error: Throwable) {
            request.errorResponse(503, "HTTP Exception: ${error.message}")
        }
    }
}

internal fun Request.errorResponse(code: Int, message: String): Response =
    Response.Builder()
        .code(code)
        .message(message)
        .request(this)
        .protocol(Protocol.HTTP_2)
        .build()
