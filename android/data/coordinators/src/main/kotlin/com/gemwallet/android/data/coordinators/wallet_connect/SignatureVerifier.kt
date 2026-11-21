package com.gemwallet.android.data.coordinators.wallet_connect

import uniffi.gemstone.GemSignatureVerifier
import java.math.BigInteger
import java.security.AlgorithmParameters
import java.security.GeneralSecurityException
import java.security.KeyFactory
import java.security.PublicKey
import java.security.Signature
import java.security.spec.ECGenParameterSpec
import java.security.spec.ECParameterSpec
import java.security.spec.ECPoint
import java.security.spec.ECPublicKeySpec

private const val UNCOMPRESSED_POINT: Byte = 0x04
private const val COORDINATE_SIZE = 32
private const val SEQUENCE: Byte = 0x30
private const val INTEGER: Byte = 0x02

class SignatureVerifier : GemSignatureVerifier {
    override fun isValidEs256(publicKey: ByteArray, message: ByteArray, signature: ByteArray): Boolean {
        if (publicKey.size != 1 + 2 * COORDINATE_SIZE || publicKey[0] != UNCOMPRESSED_POINT || signature.size != 2 * COORDINATE_SIZE) {
            return false
        }
        return try {
            Signature.getInstance("SHA256withECDSA").run {
                initVerify(publicKey(publicKey))
                update(message)
                verify(der(signature))
            }
        } catch (_: GeneralSecurityException) {
            false
        }
    }

    private fun publicKey(point: ByteArray): PublicKey {
        val curve = AlgorithmParameters.getInstance("EC")
            .apply { init(ECGenParameterSpec("secp256r1")) }
            .getParameterSpec(ECParameterSpec::class.java)
        val x = BigInteger(1, point.copyOfRange(1, 1 + COORDINATE_SIZE))
        val y = BigInteger(1, point.copyOfRange(1 + COORDINATE_SIZE, point.size))
        return KeyFactory.getInstance("EC").generatePublic(ECPublicKeySpec(ECPoint(x, y), curve))
    }

    private fun der(signature: ByteArray): ByteArray {
        val r = derInteger(signature.copyOfRange(0, COORDINATE_SIZE))
        val s = derInteger(signature.copyOfRange(COORDINATE_SIZE, signature.size))
        return byteArrayOf(SEQUENCE, (r.size + s.size).toByte()) + r + s
    }

    private fun derInteger(value: ByteArray): ByteArray {
        val bytes = BigInteger(1, value).toByteArray()
        return byteArrayOf(INTEGER, bytes.size.toByte()) + bytes
    }
}
