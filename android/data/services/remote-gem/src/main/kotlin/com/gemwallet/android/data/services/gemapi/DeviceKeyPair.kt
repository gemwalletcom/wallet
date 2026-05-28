package com.gemwallet.android.data.services.gemapi

import com.gemwallet.android.math.fromHex
import com.gemwallet.android.math.hex
import com.google.crypto.tink.subtle.Ed25519Sign
import java.security.GeneralSecurityException
import java.util.Arrays

class DeviceKeyPair private constructor(
    private val privateKey: ByteArray,
    private val publicKey: ByteArray,
) {
    val privateKeyHex: String
        get() = privateKey.hex

    val publicKeyHex: String
        get() = publicKey.hex

    fun sign(message: ByteArray): String {
        return Ed25519Sign(privateKey).sign(message).hex
    }

    companion object {
        fun generate(): DeviceKeyPair {
            val keyPair = Ed25519Sign.KeyPair.newKeyPair()
            return DeviceKeyPair(
                privateKey = keyPair.privateKey,
                publicKey = keyPair.publicKey,
            )
        }

        fun fromHex(privateKeyHex: String): DeviceKeyPair {
            val privateKey = try {
                privateKeyHex.fromHex()
            } catch (err: IllegalArgumentException) {
                throw IllegalArgumentException("Invalid device private key", err)
            }
            val seed = privateKey.copyOf()
            return try {
                DeviceKeyPair(
                    privateKey = privateKey,
                    publicKey = Ed25519Sign.KeyPair.newKeyPairFromSeed(seed).publicKey,
                )
            } catch (err: GeneralSecurityException) {
                Arrays.fill(privateKey, 0)
                throw IllegalArgumentException("Invalid device private key", err)
            } finally {
                Arrays.fill(seed, 0)
            }
        }
    }
}
