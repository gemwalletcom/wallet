package com.gemwallet.android.ext

import com.wallet.core.primitives.Transaction
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import com.wallet.core.primitives.TransactionPerpetualMetadata
import com.wallet.core.primitives.TransactionResourceTypeMetadata
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType

fun Transaction.getAssociatedAssetIds(): List<AssetId> {
    val swapAssets = getSwapMetadata()?.let { setOf(it.fromAsset, it.toAsset) } ?: emptySet()
    return (swapAssets + setOf(assetId, feeAssetId)).toList()
}

val Transaction.hash: String
    get() = id.hash

fun Transaction.getSwapMetadata(): TransactionSwapMetadata? = getTransactionSwapMetadata(type, metadata)

fun getTransactionSwapMetadata(
    type: TransactionType,
    metadata: String?,
): TransactionSwapMetadata? = decodeMetadata(type == TransactionType.Swap, metadata)

fun Transaction.getPerpetualMetadata(): TransactionPerpetualMetadata? {
    val isPerpetual = type == TransactionType.PerpetualOpenPosition ||
        type == TransactionType.PerpetualClosePosition ||
        type == TransactionType.PerpetualModifyPosition
    return decodeMetadata(isPerpetual, metadata)
}

fun Transaction.getNftMetadata(): TransactionNFTTransferMetadata? =
    decodeMetadata(type == TransactionType.TransferNFT, metadata)

fun Transaction.getResourceMetadata(): TransactionResourceTypeMetadata? {
    val isResourceTransaction = type == TransactionType.StakeFreeze || type == TransactionType.StakeUnfreeze
    return decodeMetadata(isResourceTransaction, metadata)
}

private inline fun <reified T> decodeMetadata(matches: Boolean, metadata: String?): T? {
    if (!matches || metadata.isNullOrEmpty()) {
        return null
    }
    return try {
        jsonEncoder.decodeFromString<T>(metadata)
    } catch (_: Throwable) {
        null
    }
}
