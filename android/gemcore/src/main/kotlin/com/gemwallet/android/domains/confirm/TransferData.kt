package com.gemwallet.android.domains.confirm

import com.gemwallet.android.math.fromHex
import com.gemwallet.android.math.has0xPrefix
import com.gemwallet.android.serializer.packRouteString
import com.gemwallet.android.serializer.unpackRouteString
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemTransferService

fun GemTransferService.pack(transfer: GemTransferData): String? =
    runCatching { encodeTransferData(transfer).packRouteString() }.getOrNull()

fun GemTransferService.unpack(packed: String): GemTransferData? =
    runCatching { decodeTransferData(packed.unpackRouteString()) }.getOrNull()

fun String.toTransactionData(): ByteArray =
    if (has0xPrefix()) runCatching { fromHex() }.getOrElse { toByteArray() } else toByteArray()
