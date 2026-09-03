package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.math.has0xPrefix
import com.gemwallet.android.serializer.packRouteString
import com.gemwallet.android.serializer.unpackRouteString
import com.wallet.core.primitives.Account
import com.gemwallet.android.domains.perpetual.toGem
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemTransferService

fun GemTransferData.confirmInput(from: Account): GemConfirmInput = GemConfirmInput(
    from = from.toGem(),
    transfer = this,
)

fun GemTransferService.pack(input: GemConfirmInput): String? =
    runCatching { encodeConfirmInput(input).packRouteString() }.getOrNull()

fun GemTransferService.unpack(packed: String): GemConfirmInput? =
    runCatching { decodeConfirmInput(packed.unpackRouteString()) }.getOrNull()

fun String.toTransactionData(): ByteArray =
    if (has0xPrefix()) runCatching { fromHex() }.getOrElse { toByteArray() } else toByteArray()
