package com.gemwallet.android.math

import java.security.MessageDigest

fun ByteArray.sha256Hex(): String {
    return MessageDigest.getInstance("SHA-256").digest(this).hex
}
