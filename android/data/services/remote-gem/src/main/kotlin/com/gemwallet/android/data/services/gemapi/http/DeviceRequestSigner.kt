package com.gemwallet.android.data.services.gemapi.http

import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.data.services.gemapi.DeviceKeyPair
import com.gemwallet.android.math.sha256Hex
import java.util.Base64

data class DeviceSignature(
    val authorization: String,
) {
    fun toHeaders(): Map<String, String> = mapOf(
        "Authorization" to authorization,
    )
}

class DeviceRequestSigner(
    getDeviceId: GetDeviceId,
) {
    private val deviceKeyPair = DeviceKeyPair.fromHex(getDeviceId.getDeviceKey())
    private var bodyHash: (ByteArray) -> String = { body: ByteArray ->
        body.sha256Hex()
    }
    private var signMessage: (DeviceKeyPair, ByteArray) -> String = { deviceKeyPair, message ->
        deviceKeyPair.sign(message)
    }
    private var currentTimeMillis: () -> Long = System::currentTimeMillis

    fun sign(method: String, path: String, body: ByteArray? = null, walletId: String = ""): DeviceSignature {
        val bodyHash = bodyHash(body ?: ByteArray(0))
        val timestamp = currentTimeMillis().toString()

        val message = "${timestamp}.${method}.${path}.${walletId}.${bodyHash}"
        val signatureHex = signMessage(deviceKeyPair, message.toByteArray())

        val payload = "${deviceKeyPair.publicKeyHex}.${timestamp}.${walletId}.${bodyHash}.${signatureHex}"
        val encoded = Base64.getEncoder().encodeToString(payload.toByteArray())
        return DeviceSignature(authorization = "Gem $encoded")
    }

    internal constructor(
        getDeviceId: GetDeviceId,
        bodyHash: (ByteArray) -> String = { body -> body.sha256Hex() },
        signMessage: (DeviceKeyPair, ByteArray) -> String = { deviceKeyPair, message ->
            deviceKeyPair.sign(message)
        },
        currentTimeMillis: () -> Long = System::currentTimeMillis,
    ) : this(getDeviceId) {
        this.bodyHash = bodyHash
        this.signMessage = signMessage
        this.currentTimeMillis = currentTimeMillis
    }
}
