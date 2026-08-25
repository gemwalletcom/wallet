package com.gemwallet.android.blockchain.gemstone

import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.ScanAddressTarget
import com.wallet.core.primitives.ScanTransaction
import com.wallet.core.primitives.ScanTransactionPayload
import uniffi.gemstone.ScanAddressTarget as GemScanAddressTarget
import uniffi.gemstone.ScanTransaction as GemScanTransaction
import uniffi.gemstone.ScanTransactionPayload as GemScanTransactionPayload

fun GemScanTransactionPayload.toPrimitives(): ScanTransactionPayload? {
    val origin = origin.toPrimitives() ?: return null
    val target = target.toPrimitives() ?: return null

    return ScanTransactionPayload(
        origin = origin,
        target = target,
        website = website,
        type = transactionType.toPrimitives(),
    )
}

fun ScanTransaction.toGem(): GemScanTransaction = GemScanTransaction(
    isMalicious = isMalicious,
    isMemoRequired = isMemoRequired,
)

private fun GemScanAddressTarget.toPrimitives(): ScanAddressTarget? {
    val assetId = assetId.toAssetId() ?: return null
    return ScanAddressTarget(assetId = assetId, address = address)
}
