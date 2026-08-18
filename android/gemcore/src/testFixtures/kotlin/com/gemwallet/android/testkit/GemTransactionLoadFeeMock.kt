package com.gemwallet.android.testkit

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.gemwallet.android.ext.toIdentifier
import uniffi.gemstone.GemFeeOptions
import uniffi.gemstone.GemGasPriceType
import uniffi.gemstone.GemTransactionLoadFee

fun mockGemTransactionLoadFee(
    fee: String = "500",
    gasPriceType: GemGasPriceType = GemGasPriceType.Eip1559(gasPrice = "2", priorityFee = "3"),
    feeAssetId: AssetId = mockAsset(chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18).id,
) = GemTransactionLoadFee(
    fee = fee,
    gasPriceType = gasPriceType,
    gasLimit = "21000",
    options = GemFeeOptions(emptyMap()),
    feeAsset = feeAssetId.toIdentifier(),
)
