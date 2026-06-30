package com.gemwallet.android.blockchain.gemstone

import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.model.HashChanges
import com.gemwallet.android.model.TransactionChanges
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionSwapMetadata
import uniffi.gemstone.SwapperTransactionSwapMetadata
import uniffi.gemstone.TransactionChange
import uniffi.gemstone.TransactionMetadata
import uniffi.gemstone.TransactionUpdate
import uniffi.gemstone.TransactionState as GemTransactionState

internal fun GemTransactionState.toPrimitives(): TransactionState = when (this) {
    GemTransactionState.PENDING -> TransactionState.Pending
    GemTransactionState.CONFIRMED -> TransactionState.Confirmed
    GemTransactionState.FAILED -> TransactionState.Failed
    GemTransactionState.REVERTED -> TransactionState.Reverted
    GemTransactionState.IN_TRANSIT -> TransactionState.InTransit
}

internal fun TransactionState.toGem(): GemTransactionState = when (this) {
    TransactionState.Pending -> GemTransactionState.PENDING
    TransactionState.Confirmed -> GemTransactionState.CONFIRMED
    TransactionState.Failed -> GemTransactionState.FAILED
    TransactionState.Reverted -> GemTransactionState.REVERTED
    TransactionState.InTransit -> GemTransactionState.IN_TRANSIT
}

internal fun TransactionUpdate.toPrimitives(): TransactionChanges {
    val fee = changes.firstNotNullOfOrNull { it as? TransactionChange.NetworkFee }
        ?.v1?.toBigIntegerOrNull()
    val hashChanges = changes.firstNotNullOfOrNull { it as? TransactionChange.HashChange }
    val metadata = changes.firstNotNullOfOrNull { it.swapMetadataJson() }

    return TransactionChanges(
        state = state.toPrimitives(),
        fee = fee,
        hashChanges = hashChanges?.let { HashChanges(it.old, it.new) },
        metadata = metadata,
    )
}

private fun TransactionChange.swapMetadataJson(): String? {
    val metadata = ((this as? TransactionChange.Metadata)?.v1 as? TransactionMetadata.Swap) ?: return null
    return metadata.v1.toTransactionSwapMetadata()?.toJson()
}

private fun SwapperTransactionSwapMetadata.toTransactionSwapMetadata(): TransactionSwapMetadata? {
    val fromAsset = fromAsset.toAssetId() ?: return null
    val toAsset = toAsset.toAssetId() ?: return null
    return TransactionSwapMetadata(
        fromAsset = fromAsset,
        fromValue = fromValue,
        toAsset = toAsset,
        toValue = toValue,
        provider = provider,
    )
}
