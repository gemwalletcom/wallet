package com.gemwallet.android.testkit

import com.gemwallet.android.model.Fee
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FeePriority
import java.math.BigInteger

fun mockFeeSolana(
    feeAssetId: AssetId = AssetId(Chain.Solana),
    priority: FeePriority = FeePriority.Normal,
    amount: BigInteger = BigInteger.valueOf(7_500),
    minerFee: BigInteger = BigInteger.valueOf(2_500),
    maxGasPrice: BigInteger = BigInteger.valueOf(5_000),
    unitFee: BigInteger = BigInteger.valueOf(25_000),
    limit: BigInteger = BigInteger.valueOf(100_000),
    options: Map<String, BigInteger> = emptyMap(),
) = Fee.Solana(
    feeAssetId = feeAssetId,
    priority = priority,
    amount = amount,
    minerFee = minerFee,
    maxGasPrice = maxGasPrice,
    unitFee = unitFee,
    limit = limit,
    options = options,
)
