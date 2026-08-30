package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionType
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemTransferService

class ConfirmInputProperties(
    private val transferService: GemTransferService,
) {
    fun asset(input: GemConfirmInput): Asset = transferService.asset(input.transfer.inputType).decodeJson()

    fun assetId(input: GemConfirmInput): AssetId = asset(input).id

    fun assetIds(input: GemConfirmInput): List<AssetId> =
        transferService.assetIds(input.transfer.inputType).mapNotNull { it.toAssetId() }

    fun feeAsset(input: GemConfirmInput): Asset = transferService.feeAsset(input.transfer.inputType).decodeJson()

    fun transactionType(input: GemConfirmInput): TransactionType =
        transferService.transactionType(input.transfer.inputType).decodeJson()
}
