package com.gemwallet.android.features.buy.viewmodels.models

import androidx.compose.runtime.Stable
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.models.CryptoFormattedUIModel
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatProvider
import com.wallet.core.primitives.FiatQuote
import com.wallet.core.primitives.FiatQuoteType

@Stable
data class BuyFiatProviderUIModel(
    val provider: FiatProvider,
    override val asset: Asset,
    override val cryptoAmount: Double,
    val fiatFormatted: String,
    val rate: String,
) : CryptoFormattedUIModel {

    override val cryptoFormatted: String
        get() = "≈ ${super.cryptoFormatted}"

    val cryptoText: String
        get() = super.cryptoFormatted
}

fun FiatQuote.toProviderUIModel(
    asset: Asset,
    currency: Currency,
    assetPrice: Double? = null,
): BuyFiatProviderUIModel {
    val formatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = currency)
    return BuyFiatProviderUIModel(
        provider = provider,
        asset = asset,
        cryptoAmount = cryptoAmount,
        fiatFormatted = formatter.string(displayFiatAmount(assetPrice)),
        rate = "1 ${asset.symbol} ≈ ${formatter.string(fiatAmount / cryptoAmount)}",
    )
}

private fun FiatQuote.displayFiatAmount(assetPrice: Double?): Double = when (type) {
    FiatQuoteType.Buy -> assetPrice?.takeIf { it > 0.0 }?.let { it * cryptoAmount } ?: fiatAmount
    FiatQuoteType.Sell -> fiatAmount
}


