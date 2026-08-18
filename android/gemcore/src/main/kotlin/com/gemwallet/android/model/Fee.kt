package com.gemwallet.android.model

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FeePriority
import java.math.BigInteger

sealed class Fee(
    val feeAssetId: AssetId,
    val priority: FeePriority,
    val amount: BigInteger,
    val options: Map<String, BigInteger>,
) {

    class Plain(
        feeAssetId: AssetId,
        priority: FeePriority,
        amount: BigInteger,
        options: Map<String, BigInteger>,
    ) : Fee(feeAssetId, priority, amount, options)

    class Regular(
        feeAssetId: AssetId,
        priority: FeePriority,
        amount: BigInteger,
        val maxGasPrice: BigInteger,
        val limit: BigInteger,
        options: Map<String, BigInteger>,
    ) : Fee(feeAssetId, priority, amount, options)

    class Eip1559(
        feeAssetId: AssetId,
        priority: FeePriority,
        amount: BigInteger,
        val maxGasPrice: BigInteger,
        val minerFee: BigInteger,
        val limit: BigInteger,
        options: Map<String, BigInteger>,
    ) : Fee(feeAssetId, priority, amount, options)

    class Solana(
        feeAssetId: AssetId,
        priority: FeePriority,
        amount: BigInteger,
        val minerFee: BigInteger,
        val maxGasPrice: BigInteger,
        val unitFee: BigInteger,
        val limit: BigInteger,
        options: Map<String, BigInteger>,
    ) : Fee(feeAssetId, priority, amount, options)
}
