package com.gemwallet.android.blockchain.operators.gemstone

import uniffi.gemstone.GemKeystore

internal inline fun <R> withGemKeystore(
    baseDir: String,
    password: String,
    block: (keystore: GemKeystore, passwordBytes: ByteArray) -> R,
): R {
    require(password.isNotEmpty()) { "keystore password is missing" }
    return GemKeystore(baseDir).use { keystore ->
        val passwordBytes = keystore.decodePassword(password)
        try {
            block(keystore, passwordBytes)
        } finally {
            passwordBytes.fill(0)
        }
    }
}
