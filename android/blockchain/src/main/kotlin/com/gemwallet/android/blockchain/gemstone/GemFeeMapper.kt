package com.gemwallet.android.blockchain.gemstone

import com.gemwallet.android.ext.toChainType
import com.gemwallet.android.ext.toFeePriority
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.Fee
import com.gemwallet.android.model.FeeSelection
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainType
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.customGasPrice
import uniffi.gemstone.FeeOption as GemFeeOption
import uniffi.gemstone.GemFeeOptions
import uniffi.gemstone.GemGasPriceType
import uniffi.gemstone.GemTransactionLoadFee

internal fun Fee.toGemGasPriceType(): GemGasPriceType = when (this) {
    is Fee.Eip1559 -> GemGasPriceType.Eip1559(
        gasPrice = maxGasPrice.toString(),
        priorityFee = minerFee.toString(),
    )
    is Fee.Plain -> GemGasPriceType.Regular(
        gasPrice = amount.toString(),
    )
    is Fee.Regular -> GemGasPriceType.Regular(
        gasPrice = maxGasPrice.toString(),
    )
    is Fee.Solana -> GemGasPriceType.Solana(
        gasPrice = maxGasPrice.toString(),
        priorityFee = minerFee.toString(),
        unitPrice = unitFee.toString(),
    )
}

internal fun Fee.toGemSignerFee(): GemTransactionLoadFee = GemTransactionLoadFee(
    fee = amount.toString(),
    gasPriceType = toGemGasPriceType(),
    gasLimit = when (this) {
        is Fee.Eip1559 -> limit.toString()
        is Fee.Regular -> limit.toString()
        is Fee.Solana -> limit.toString()
        is Fee.Plain -> "0"
    },
    options = GemFeeOptions(
        options.mapNotNull { (key, value) ->
            runCatching { GemFeeOption.valueOf(key) }.getOrNull()?.let { option ->
                option to value.toString()
            }
        }.toMap()
    ),
    feeAsset = feeAssetId.toIdentifier(),
)

internal fun List<GemFeeRate>.selectFeeRate(selection: FeeSelection): GemFeeRate = when (selection) {
    is FeeSelection.Preset -> firstOrNull { it.priority.toFeePriority() == selection.priority }
        ?: firstOrNull()
        ?: throw IllegalStateException("Fee rates not found")

    is FeeSelection.Custom -> {
        val base = firstOrNull { it.priority.toFeePriority() == FeePriority.Normal }
            ?: firstOrNull()
            ?: throw IllegalStateException("Fee rates not found")
        GemFeeRate(base.priority, customGasPrice(base.gasPriceType, selection.gasPrice.toString()))
    }
}

internal fun GemTransactionLoadFee.toFee(
    priority: FeePriority,
    feeAssetId: AssetId,
): Fee {
    return when (feeAssetId.chain.toChainType()) {
        ChainType.Solana -> toSolanaFee(feeAssetId, priority)
        ChainType.Bitcoin,
        ChainType.Cosmos,
        ChainType.Tron,
        ChainType.Aptos -> toRegularFee(feeAssetId, priority)
        ChainType.Ethereum -> toEip1559Fee(feeAssetId, priority)
        ChainType.HyperCore,
        ChainType.Ton,
        ChainType.Sui,
        ChainType.Xrp,
        ChainType.Near,
        ChainType.Stellar,
        ChainType.Algorand,
        ChainType.Polkadot,
        ChainType.Cardano -> toPlainFee(feeAssetId, priority)
    }
}

private fun GemTransactionLoadFee.toFeeOptions() = options.options
    .mapKeys { it.key.name }
    .mapValues { it.value.toBigInteger() }

private fun GemTransactionLoadFee.toPlainFee(feeAssetId: AssetId, priority: FeePriority): Fee.Plain {
    return Fee.Plain(
        feeAssetId = feeAssetId,
        priority = priority,
        amount = fee.toBigInteger(),
        options = toFeeOptions(),
    )
}

private fun GemTransactionLoadFee.toRegularFee(feeAssetId: AssetId, priority: FeePriority): Fee.Regular {
    val price = gasPriceType as GemGasPriceType.Regular
    return Fee.Regular(
        feeAssetId = feeAssetId,
        priority = priority,
        maxGasPrice = price.gasPrice.toBigInteger(),
        limit = gasLimit.toBigInteger(),
        amount = fee.toBigInteger(),
        options = toFeeOptions(),
    )
}

private fun GemTransactionLoadFee.toEip1559Fee(feeAssetId: AssetId, priority: FeePriority): Fee.Eip1559 {
    val price = gasPriceType as GemGasPriceType.Eip1559
    return Fee.Eip1559(
        feeAssetId = feeAssetId,
        priority = priority,
        maxGasPrice = price.gasPrice.toBigInteger(),
        minerFee = price.priorityFee.toBigInteger(),
        limit = gasLimit.toBigInteger(),
        amount = fee.toBigInteger(),
        options = toFeeOptions(),
    )
}

private fun GemTransactionLoadFee.toSolanaFee(feeAssetId: AssetId, priority: FeePriority): Fee.Solana {
    val price = gasPriceType as GemGasPriceType.Solana
    return Fee.Solana(
        feeAssetId = feeAssetId,
        priority = priority,
        maxGasPrice = price.gasPrice.toBigInteger(),
        minerFee = price.priorityFee.toBigInteger(),
        unitFee = price.unitPrice.toBigInteger(),
        limit = gasLimit.toBigInteger(),
        amount = fee.toBigInteger(),
        options = toFeeOptions(),
    )
}
