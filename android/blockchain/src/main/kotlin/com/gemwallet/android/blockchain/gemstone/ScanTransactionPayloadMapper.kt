package com.gemwallet.android.blockchain.gemstone

import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.ScanAddressTarget
import com.wallet.core.primitives.ScanTransaction
import com.wallet.core.primitives.ScanTransactionPayload

internal fun ConfirmParams.toScanTransactionPayload(destination: String): ScanTransactionPayload = ScanTransactionPayload(
    origin = ScanAddressTarget(
        assetId = assetId,
        address = from.address,
    ),
    target = ScanAddressTarget(
        assetId = if (this is ConfirmParams.SwapParams) toAsset.id else assetId,
        address = destination,
    ),
    website = (this as? ConfirmParams.TransferParams.Generic)?.metadata?.url,
    type = getTransactionType(),
)

internal fun ScanTransaction.validate(params: ConfirmParams) {
    if (isMalicious) {
        throw ConfirmError.ScanTransactionMalicious
    }
    if (isMemoRequired && params.memo().isNullOrBlank()) {
        throw ConfirmError.ScanTransactionMemoRequired(params.asset.symbol)
    }
}
