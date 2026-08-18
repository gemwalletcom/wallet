package com.gemwallet.android.testkit

import com.gemwallet.android.model.Fee
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FeePriority
import java.math.BigInteger

fun mockFeeSolana(
    feeAsset: Asset = mockAssetSolana(),
    priority: FeePriority = FeePriority.Normal,
    amount: BigInteger = BigInteger.valueOf(7_500),
    minerFee: BigInteger = BigInteger.valueOf(2_500),
    maxGasPrice: BigInteger = BigInteger.valueOf(5_000),
    unitFee: BigInteger = BigInteger.valueOf(25_000),
    limit: BigInteger = BigInteger.valueOf(100_000),
    options: Map<String, BigInteger> = emptyMap(),
) = Fee.Solana(
    feeAsset = feeAsset,
    priority = priority,
    amount = amount,
    minerFee = minerFee,
    maxGasPrice = maxGasPrice,
    unitFee = unitFee,
    limit = limit,
    options = options,
)
