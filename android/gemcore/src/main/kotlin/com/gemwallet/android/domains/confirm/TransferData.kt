package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.math.has0xPrefix
import com.gemwallet.android.serializer.packRouteString
import com.gemwallet.android.serializer.unpackRouteString
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualType
import com.wallet.core.primitives.StakeType
import com.gemwallet.android.domains.perpetual.toGem
import com.wallet.core.primitives.PerpetualProvider
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemTransferService
import java.math.BigInteger

fun GemTransferData.confirmInput(from: Account): GemConfirmInput = GemConfirmInput(
    from = from.toGem(),
    transfer = this,
)

fun GemTransferService.pack(input: GemConfirmInput): String? =
    runCatching { encodeConfirmInput(input).packRouteString() }.getOrNull()

fun GemTransferService.unpack(packed: String): GemConfirmInput? =
    runCatching { decodeConfirmInput(packed.unpackRouteString()) }.getOrNull()

fun GemTransferData.Companion.perpetual(
    asset: Asset,
    perpetualType: PerpetualType,
    value: BigInteger = BigInteger.ZERO,
    useMaxAmount: Boolean = false,
): GemTransferData = GemPerpetual(PerpetualProvider.Hypercore.toGem()).use {
    it.transferData(asset.toGem(), perpetualType.toGem(), value.toString(), useMaxAmount)
}

fun String.toTransactionData(): ByteArray =
    if (has0xPrefix()) runCatching { fromHex() }.getOrElse { toByteArray() } else toByteArray()
