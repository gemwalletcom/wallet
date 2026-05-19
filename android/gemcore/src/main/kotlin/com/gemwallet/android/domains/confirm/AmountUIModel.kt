package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.TransactionType
import java.math.BigInteger

class AmountUIModel(
    val txType: TransactionType,
    val amount: BigInteger,
    val asset: AssetInfo,
    val fromAsset: AssetInfo,
    val toAsset: AssetInfo?,
    val fromAmount: String,
    val toAmount: String?,
    val nftAsset: NFTAsset?,
    val price: Double?,
    val currency: Currency = Currency.USD,
) {
    val cryptoAmount: String by lazy {
        ValueFormatter(style = ValueFormatter.Style.Full)
            .string(amount, asset.asset.decimals, asset.asset.symbol)
    }

    val amountEquivalent: String by lazy {
        if (price == null) ""
        else CurrencyFormatter(currency = currency)
            .string(CryptoFiatConverter.toFiat(Crypto(amount), asset.asset.decimals, price).atomicValue)
    }
}
