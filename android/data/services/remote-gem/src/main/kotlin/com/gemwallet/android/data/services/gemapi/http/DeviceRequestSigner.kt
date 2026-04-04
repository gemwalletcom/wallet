package com.gemwallet.android.data.services.gemapi.http

import android.util.Base64
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.math.decodeHex
import com.gemwallet.android.math.toHexString
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
    fun sign(method: String, path: String, body: ByteArray? = null, walletId: String = ""): DeviceSignature {
        val privateKey = PrivateKey(getDeviceId.getDeviceKey().decodeHex())
        val publicKeyHex = getDeviceId.getDeviceId()
        val bodyHash = Hash.sha256(body ?: ByteArray(0)).toHexString("")
        val timestamp = System.currentTimeMillis().toString()

        val message = "${timestamp}.${method}.${path}.${walletId}.${bodyHash}"
        val signatureBytes = privateKey.sign(message.toByteArray(), Curve.ED25519)
        val signatureHex = signatureBytes.toHexString("")

        val payload = "${publicKeyHex}.${timestamp}.${walletId}.${bodyHash}.${signatureHex}"
        val encoded = Base64.encodeToString(payload.toByteArray(), Base64.NO_WRAP)
        return DeviceSignature(authorization = "Gem $encoded")
    }
}
