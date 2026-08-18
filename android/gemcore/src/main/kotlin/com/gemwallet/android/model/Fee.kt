package com.gemwallet.android.model

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FeePriority
import java.math.BigInteger

sealed interface Fee {

    val priority: FeePriority
    val feeAsset: Asset
    val feeAssetId: AssetId get() = feeAsset.id

    val amount: BigInteger

    class Plain(
        override val feeAsset: Asset,
        override val priority: FeePriority,
        override val amount: BigInteger,
        val options: Map<String, BigInteger>,
    ) : Fee

    class Regular(
        override val feeAsset: Asset,
        override val priority: FeePriority,
        override val amount: BigInteger,
        val maxGasPrice: BigInteger,
        val limit: BigInteger,
        val options: Map<String, BigInteger>,
    ) : Fee

    class Eip1559(
        override val feeAsset: Asset,
        override val priority: FeePriority,
        override val amount: BigInteger,
        val maxGasPrice: BigInteger,
        val minerFee: BigInteger,
        val limit: BigInteger,
        val options: Map<String, BigInteger>,
    ) : Fee

    class Solana(
        override val feeAsset: Asset,
        override val priority: FeePriority,
        override val amount: BigInteger,
        val minerFee: BigInteger,
        val maxGasPrice: BigInteger,
        val unitFee: BigInteger,
        val limit: BigInteger,
        val options: Map<String, BigInteger>,
    ) : Fee
}
