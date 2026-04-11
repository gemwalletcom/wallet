package com.gemwallet.android.data.services.gemapi.http

import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.math.decodeHex
import com.gemwallet.android.math.toHexString
import java.util.Base64
import wallet.core.jni.Curve
import wallet.core.jni.Hash
import wallet.core.jni.PrivateKey

data class DeviceSignature(
    val authorization: String,
) {
    fun toHeaders(): Map<String, String> = mapOf(
        "Authorization" to authorization,
    )
}

class DeviceRequestSigner(
    private val getDeviceId: GetDeviceId,
) {
    private var bodyHash: (ByteArray) -> String = { body: ByteArray ->
        Hash.sha256(body).toHexString("")
    }
    private var signMessage: (String, ByteArray) -> String = { privateKeyHex: String, message: ByteArray ->
        PrivateKey(privateKeyHex.decodeHex()).sign(message, Curve.ED25519).toHexString("")
    }
    private var currentTimeMillis: () -> Long = System::currentTimeMillis

    fun sign(method: String, path: String, body: ByteArray? = null, walletId: String = ""): DeviceSignature {
        val publicKeyHex = getDeviceId.getDeviceId()
        val bodyHash = bodyHash(body ?: ByteArray(0))
        val timestamp = currentTimeMillis().toString()

        val message = "${timestamp}.${method}.${path}.${walletId}.${bodyHash}"
        val signatureHex = signMessage(getDeviceId.getDeviceKey(), message.toByteArray())

        val payload = "${publicKeyHex}.${timestamp}.${walletId}.${bodyHash}.${signatureHex}"
        val encoded = Base64.getEncoder().encodeToString(payload.toByteArray())
        return DeviceSignature(authorization = "Gem $encoded")
    }

    internal constructor(
        getDeviceId: GetDeviceId,
        bodyHash: (ByteArray) -> String,
        signMessage: (String, ByteArray) -> String,
        currentTimeMillis: () -> Long,
    ) : this(getDeviceId) {
        this.bodyHash = bodyHash
        this.signMessage = signMessage
        this.currentTimeMillis = currentTimeMillis
    }
}
