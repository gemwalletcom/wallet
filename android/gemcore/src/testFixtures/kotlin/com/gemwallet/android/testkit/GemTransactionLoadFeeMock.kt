package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemFeeOptions
import uniffi.gemstone.GemGasPriceType
import uniffi.gemstone.GemTransactionLoadFee

fun mockGemTransactionLoadFee(
    fee: String = "500",
    gasPriceType: GemGasPriceType = GemGasPriceType.Eip1559(gasPrice = "2", priorityFee = "3"),
    feeAssetId: AssetId = AssetId(Chain.Ethereum),
) = GemTransactionLoadFee(
    fee = fee,
    gasPriceType = gasPriceType,
    gasLimit = "21000",
    options = GemFeeOptions(emptyMap()),
    feeAssetId = feeAssetId.toIdentifier(),
)
