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
    val row: GemFiatQuoteRow,
    override val asset: Asset,
    val currency: Currency,
) : CryptoFormattedUIModel {

    val provider: FiatProviderName by lazy { row.provider.toPrimitives() }

    val providerName: String get() = row.providerName

    val providerImageUrl: String? get() = row.providerImageUrl

    override val cryptoAmount: Double get() = row.cryptoAmount

    override val cryptoFormatted: String by lazy { "≈ $cryptoText" }

    val cryptoText: String by lazy {
        ValueFormatter(style = ValueFormatter.Style.Auto).string(BigDecimal.valueOf(cryptoAmount), asset.symbol)
    }

    val fiatFormatted: String by lazy { fiatFormatter.string(row.fiatAmount) }

    val rate: String by lazy {
        row.rate?.let { "1 ${it.baseSymbol} ≈ ${fiatFormatter.string(it.value)}" }.orEmpty()
    }

    private val fiatFormatter: CurrencyFormatter
        get() = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = currency)
}

fun GemFiatQuoteRow.toProviderUIModel(asset: Asset, currency: Currency): BuyFiatProviderUIModel =
    BuyFiatProviderUIModel(row = this, asset = asset, currency = currency)
