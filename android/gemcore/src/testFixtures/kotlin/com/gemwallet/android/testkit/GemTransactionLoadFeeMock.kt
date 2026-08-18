package com.gemwallet.android.testkit

import com.gemwallet.android.domains.asset.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemFeeOptions
import uniffi.gemstone.GemGasPriceType
import uniffi.gemstone.GemTransactionLoadFee

fun mockGemTransactionLoadFee(
    fee: String = "500",
    gasPriceType: GemGasPriceType = GemGasPriceType.Eip1559(gasPrice = "2", priorityFee = "3"),
    feeAsset: Asset = mockAsset(chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18),
) = GemTransactionLoadFee(
    fee = fee,
    gasPriceType = gasPriceType,
    gasLimit = "21000",
    options = GemFeeOptions(emptyMap()),
    feeAsset = feeAsset.toGem(),
)
