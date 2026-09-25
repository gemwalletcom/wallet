package com.gemwallet.android.testkit

import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.FeeOption
import uniffi.gemstone.GemFeeOptionItem
import uniffi.gemstone.feeAmount
import java.math.BigInteger

fun mockFeeInfo(amount: BigInteger = BigInteger("1000"), feeAsset: Asset = mockAssetEthereum(), price: Double? = null, additionalFees: List<Pair<FeeOption, BigInteger>> = emptyList()): FeeUIModel.FeeInfo {
    val formatted = { value: BigInteger -> feeAmount(feeAsset.toGem(), value, price, Currency.USD.toGem()) }
    return FeeUIModel.FeeInfo(
        amount = amount,
        feeAsset = feeAsset,
        price = price,
        currency = Currency.USD,
        priority = FeePriority.Normal,
        display = formatted(amount),
        additionalFees = additionalFees.map { (option, value) -> GemFeeOptionItem(option, value, formatted(value)) },
    )
}
