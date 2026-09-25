package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.text
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.FeeOption
import uniffi.gemstone.GemFeeAmount
import uniffi.gemstone.GemFeeOptionItem
import java.math.BigInteger

sealed interface FeeUIModel {
    data object Calculating : FeeUIModel
    data class Unavailable(val text: String) : FeeUIModel
    class FeeInfo(val amount: BigInteger, val feeAsset: Asset, val price: Double?, val currency: Currency, val priority: FeePriority, private val display: GemFeeAmount, additionalFees: List<GemFeeOptionItem> = emptyList()) : FeeUIModel {
        val feeItems: List<Pair<FeeOption, FeeInfo>> by lazy {
            additionalFees.map { it.option to FeeInfo(it.value, feeAsset, price, currency, priority, it.amount) }
        }

        val cryptoAmount: String by lazy { display.amount.text() }

        val fiatAmount: String by lazy { display.fiat?.text().orEmpty() }
    }
}
