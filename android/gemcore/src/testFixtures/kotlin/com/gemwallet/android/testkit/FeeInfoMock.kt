package com.gemwallet.android.testkit

import com.gemwallet.android.domains.confirm.FeeUIModel
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemFeeOptionItem
import java.math.BigInteger

fun mockFeeInfo(
    amount: BigInteger = BigInteger("1000"),
    feeAsset: Asset = mockAssetEthereum(),
    price: Double? = null,
    additionalFees: List<GemFeeOptionItem> = emptyList(),
) = FeeUIModel.FeeInfo(
    amount = amount,
    feeAsset = feeAsset,
    price = price,
    currency = Currency.USD,
    priority = FeePriority.Normal,
    additionalFees = additionalFees,
)
