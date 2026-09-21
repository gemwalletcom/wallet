package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.NFTAsset
import uniffi.gemstone.GemTransactionHeaderKind
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.PaymentPrice

import java.math.BigInteger

class AmountUIModel(
    val headerKind: GemTransactionHeaderKind,
    val amount: BigInteger,
    val fromAsset: AssetPriceValue,
    val toAsset: AssetPriceValue?,
    val fromAmount: BigInteger,
    val toAmount: BigInteger?,
    val nftAsset: NFTAsset?,
    val currency: Currency,
    val paymentPrice: PaymentPrice?,
) {
    val asset: Asset get() = fromAsset.asset

    val fromAmountText: String get() = ValueFormatter(style = GemValueStyle.AUTO).string(fromAmount, fromAsset.asset)

    val toAmountText: String? get() = toAmount?.let { amount -> toAsset?.let { ValueFormatter(style = GemValueStyle.AUTO).string(amount, it.asset) } }

    val fromAmountEquivalentText: String? get() = fromAsset.price?.price?.price?.let { CryptoFiatConverter.toFiatString(Crypto(fromAmount), fromAsset.asset.decimals, it, currency) }

    val toAmountEquivalentText: String? get() = toAmount?.let { amount -> toAsset?.let { asset -> asset.price?.price?.price?.let { CryptoFiatConverter.toFiatString(Crypto(amount), asset.asset.decimals, it, currency) } } }

    val cryptoAmount: String by lazy {
        ValueFormatter(style = GemValueStyle.FULL)
            .string(amount, asset.decimals, asset.symbol)
    }

    val amountEquivalent: String by lazy {
        paymentPrice?.let { return@lazy CurrencyFormatter(currencyCode = it.currency).string(it.amount) }
        val price = fromAsset.price?.price?.price ?: return@lazy ""
        CryptoFiatConverter.toFiatString(Crypto(amount), asset.decimals, price, currency)
    }
}
