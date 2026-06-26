package com.gemwallet.android.data.services.gemapi.http

import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.math.fromHex
import kotlinx.coroutines.runBlocking
import uniffi.gemstone.signDeviceAuth

data class DeviceSignature(
    val authorization: String,
) {
    fun toHeaders(): Map<String, String> = mapOf(
        "Authorization" to authorization,
    )
}

interface DeviceRequestSigner {
    fun sign(method: String, path: String, body: ByteArray? = null, walletId: String = ""): DeviceSignature
}

class GemDeviceRequestSigner(
    getDeviceId: GetDeviceId,
) : DeviceRequestSigner {
    private val privateKey by lazy(LazyThreadSafetyMode.SYNCHRONIZED) {
        runBlocking { getDeviceId.getDeviceKey().fromHex() }
    }

    override fun sign(method: String, path: String, body: ByteArray?, walletId: String): DeviceSignature =
        DeviceSignature(signDeviceAuth(privateKey, method, path, walletId, body ?: ByteArray(0), System.currentTimeMillis().toULong()))
}
