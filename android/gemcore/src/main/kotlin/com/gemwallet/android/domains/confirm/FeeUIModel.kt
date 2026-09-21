package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.FeeOption
import uniffi.gemstone.GemFeeOptionItem
import uniffi.gemstone.GemValueStyle
import java.math.BigInteger

sealed interface FeeUIModel {
    data object Calculating : FeeUIModel
    data class Unavailable(val text: String) : FeeUIModel
    class FeeInfo(val amount: BigInteger, val feeAsset: Asset, val price: Double?, val currency: Currency, val priority: FeePriority, additionalFees: List<GemFeeOptionItem> = emptyList()) : FeeUIModel {
        val feeItems: List<Pair<FeeOption, FeeInfo>> by lazy {
            additionalFees.map { it.option to FeeInfo(it.value, feeAsset, price, currency, priority) }
        }

        val cryptoAmount: String by lazy {
            ValueFormatter(style = GemValueStyle.AUTO).string(amount, feeAsset)
        }

        val fiatAmount: String by lazy {
            if (price == null) {
                ""
            } else {
                CryptoFiatConverter.toFiatString(Crypto(amount), feeAsset.decimals, price, currency)
            }
        }

        val cryptoAmountWithFiat: String by lazy {
            if (fiatAmount.isEmpty()) cryptoAmount else "$cryptoAmount (~$fiatAmount)"
        }
    }
}
