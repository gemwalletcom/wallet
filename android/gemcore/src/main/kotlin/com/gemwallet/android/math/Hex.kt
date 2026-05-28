package com.gemwallet.android.math

import java.math.BigInteger

private const val HEX_CHARS = "0123456789abcdef"

fun Byte.toHex(): String {
    val i = toInt()
    return HEX_CHARS[i.shr(4) and 0x0f].toString() + HEX_CHARS[i and 0x0f].toString()
}

val ByteArray.hex: String
    get() = joinToString("") { it.toHex() }

fun String.has0xPrefix() = startsWith("0x")

fun String.remove0x() = if (has0xPrefix()) substring(2) else this

fun Char.hexToBin(): Int = when (this) {
    in '0' .. '9' -> this - '0'
    in 'A' .. 'F' -> this - 'A' + 10
    in 'a' .. 'f' -> this - 'a' + 10
    else -> throw IllegalArgumentException("$this is not valid hex char")
}

fun String.fromHex(): ByteArray {
    val value = remove0x()
    if ((value.length % 2) != 0) {
        throw IllegalArgumentException("hex-string must have even number of digits")
    }
    return ByteArray(value.length / 2).apply {
        var i = 0
        while (i < value.length) {
            this[i / 2] = ((value[i].hexToBin() shl 4) + value[i + 1].hexToBin()).toByte()
            i += 2
        }
    }
}

fun String.append0x(): String = if (startsWith("0x")) this else "0x$this"

fun String.hexToBigInteger(): BigInteger? = try {
    if (has0xPrefix()) BigInteger(remove0x(), 16) else BigInteger(this)
} catch (_: NumberFormatException) {
    null
}
