package com.gemwallet.android.ext

import com.wallet.core.primitives.Transaction
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import com.wallet.core.primitives.TransactionPerpetualMetadata
import com.wallet.core.primitives.TransactionResourceTypeMetadata
import com.wallet.core.primitives.TransferDataOutputAction
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import kotlinx.serialization.Serializable

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
): TransactionSwapMetadata? {
    if (type != TransactionType.Swap ||  metadata.isNullOrEmpty()) {
        return null
    }
    return try {
        jsonEncoder.decodeFromString(metadata)
    } catch (_: Throwable) {
        null
    }
}

fun Transaction.getPerpetualMetadata(): TransactionPerpetualMetadata? {
    val isPerpetual = type == TransactionType.PerpetualOpenPosition ||
        type == TransactionType.PerpetualClosePosition ||
        type == TransactionType.PerpetualModifyPosition
    if (!isPerpetual || metadata.isNullOrEmpty()) return null
    return try {
        jsonEncoder.decodeFromString(metadata)
    } catch (_: Throwable) {
        null
    }
}

fun Transaction.getNftMetadata(): TransactionNFTTransferMetadata? {
    if (type != TransactionType.TransferNFT || metadata.isNullOrEmpty()) {
        return null
    }
    return try {
        jsonEncoder.decodeFromString(metadata)
    } catch (_: Throwable) {
        null
    }
}

fun Transaction.getResourceMetadata(): TransactionResourceTypeMetadata? {
    val isResourceTransaction = type == TransactionType.StakeFreeze || type == TransactionType.StakeUnfreeze
    if (!isResourceTransaction || metadata.isNullOrEmpty()) {
        return null
    }
    return try {
        jsonEncoder.decodeFromString(metadata)
    } catch (_: Throwable) {
        null
    }
}

fun Transaction.getWalletConnectOutputAction(): TransferDataOutputAction? {
    if (type != TransactionType.SmartContractCall || metadata.isNullOrEmpty()) {
        return null
    }

    return try {
        jsonEncoder.decodeFromString<TransactionWalletConnectMetadata>(metadata).outputAction
    } catch (_: Throwable) {
        null
    }
}

@Serializable
private data class TransactionWalletConnectMetadata(
    val outputAction: TransferDataOutputAction,
)
