package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import java.math.BigInteger

sealed interface FeeUIModel {
    data object Calculating : FeeUIModel
    data object Error : FeeUIModel
    class FeeInfo(
        val amount: BigInteger,
        val feeAsset: Asset,
        val price: Double?,
        val currency: Currency,
        val priority: FeePriority,
    ) : FeeUIModel {
        val cryptoAmount: String by lazy {
            ValueFormatter(style = ValueFormatter.Style.Full).string(amount, feeAsset)
        }

        val fiatAmount: String by lazy {
            if (price == null) ""
            else CurrencyFormatter(currency = currency)
                .string(CryptoFiatConverter.toFiat(Crypto(amount), feeAsset.decimals, price).atomicValue)
        }
    }
}
