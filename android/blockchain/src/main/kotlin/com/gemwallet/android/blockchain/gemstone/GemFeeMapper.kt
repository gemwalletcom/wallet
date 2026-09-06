package com.gemwallet.android.blockchain.gemstone

import com.gemwallet.android.ext.toChainType
import com.gemwallet.android.model.Fee
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainType
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GasPriceType
import uniffi.gemstone.GemTransactionLoadFee

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

private fun GemTransactionLoadFee.toFeeOptions() = options.options.mapKeys { it.key.name }

private fun GemTransactionLoadFee.toPlainFee(feeAssetId: AssetId, priority: FeePriority): Fee.Plain {
    return Fee.Plain(
        feeAssetId = feeAssetId,
        priority = priority,
        amount = fee,
        options = toFeeOptions(),
    )
}

private fun GemTransactionLoadFee.toRegularFee(feeAssetId: AssetId, priority: FeePriority): Fee.Regular {
    val price = gasPriceType as GasPriceType.Regular
    return Fee.Regular(
        feeAssetId = feeAssetId,
        priority = priority,
        maxGasPrice = price.gasPrice,
        limit = gasLimit,
        amount = fee,
        options = toFeeOptions(),
    )
}

private fun GemTransactionLoadFee.toEip1559Fee(feeAssetId: AssetId, priority: FeePriority): Fee.Eip1559 {
    val price = gasPriceType as GasPriceType.Eip1559
    return Fee.Eip1559(
        feeAssetId = feeAssetId,
        priority = priority,
        maxGasPrice = price.gasPrice,
        minerFee = price.priorityFee,
        limit = gasLimit,
        amount = fee,
        options = toFeeOptions(),
    )
}

private fun GemTransactionLoadFee.toSolanaFee(feeAssetId: AssetId, priority: FeePriority): Fee.Solana {
    val price = gasPriceType as GasPriceType.Solana
    return Fee.Solana(
        feeAssetId = feeAssetId,
        priority = priority,
        maxGasPrice = price.gasPrice,
        minerFee = price.priorityFee,
        unitFee = price.unitPrice,
        limit = gasLimit,
        amount = fee,
        options = toFeeOptions(),
    )
}
