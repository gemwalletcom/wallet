package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.TransactionType
import uniffi.gemstone.GemTransactionHeaderKind
import java.math.BigInteger

class AmountUIModel(
    val transactionType: TransactionType,
    val headerKind: GemTransactionHeaderKind,
    val amount: BigInteger,
    val fromAsset: AssetPriceValue,
    val toAsset: AssetPriceValue?,
    val fromAmount: String,
    val toAmount: String?,
    val nftAsset: NFTAsset?,
    val currency: Currency,
) {
    val asset: Asset get() = fromAsset.asset

    val cryptoAmount: String by lazy {
        ValueFormatter(style = ValueFormatter.Style.Full)
            .string(amount, asset.decimals, asset.symbol)
    }

    val amountEquivalent: String by lazy {
        val price = fromAsset.price?.price?.price ?: return@lazy ""
        CryptoFiatConverter.toFiatString(Crypto(amount), asset.decimals, price, currency)
    }
}
