package com.gemwallet.android.features.buy.viewmodels.models

import androidx.compose.runtime.Stable
import com.gemwallet.android.model.format
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
) = BuyFiatProviderUIModel(
    provider = provider,
    asset = asset,
    cryptoAmount = cryptoAmount,
    fiatFormatted = currency.format(displayFiatAmount(assetPrice)),
    rate = asset.rateText(fiatAmount, cryptoAmount, currency),
)

private fun FiatQuote.displayFiatAmount(assetPrice: Double?): Double = when (type) {
    FiatQuoteType.Buy -> assetPrice?.takeIf { it > 0.0 }?.let { it * cryptoAmount } ?: fiatAmount
    FiatQuoteType.Sell -> fiatAmount
}

private fun Asset.rateText(fiatAmount: Double, cryptoAmount: Double, currency: Currency) =
    "1 $symbol ≈ ${currency.format(fiatAmount / cryptoAmount).format(currency.string, 2)}"


