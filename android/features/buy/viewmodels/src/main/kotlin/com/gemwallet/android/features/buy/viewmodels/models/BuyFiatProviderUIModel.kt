package com.gemwallet.android.features.buy.viewmodels.models

import androidx.compose.runtime.Stable
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.models.CryptoFormattedUIModel
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatProviderName
import java.math.BigDecimal
import uniffi.gemstone.GemFiatQuoteRow

@Stable
data class BuyFiatProviderUIModel(
    val provider: FiatProviderName,
    val providerName: String,
    val providerImageUrl: String?,
    override val asset: Asset,
    override val cryptoAmount: Double,
    val fiatFormatted: String,
    val rate: String,
) : CryptoFormattedUIModel {

    override val cryptoFormatted: String by lazy { "≈ $cryptoText" }

    val cryptoText: String by lazy {
        ValueFormatter(style = ValueFormatter.Style.Auto).string(BigDecimal.valueOf(cryptoAmount), asset.symbol)
    }
}

fun GemFiatQuoteRow.toProviderUIModel(asset: Asset, currency: Currency): BuyFiatProviderUIModel {
    val formatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = currency)
    return BuyFiatProviderUIModel(
        provider = provider.toPrimitives(),
        providerName = providerName,
        providerImageUrl = providerImageUrl,
        asset = asset,
        cryptoAmount = cryptoAmount,
        fiatFormatted = formatter.string(fiatAmount),
        rate = rate?.let { "1 ${asset.symbol} ≈ ${formatter.string(it)}" }.orEmpty(),
    )
}
