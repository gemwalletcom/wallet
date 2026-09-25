package com.gemwallet.android.features.buy.viewmodels.models

import androidx.compose.runtime.Stable
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.wallet.core.primitives.FiatProviderName
import uniffi.gemstone.GemFiatQuoteRow

@Stable
data class BuyFiatProviderUIModel(val row: GemFiatQuoteRow) {

    val provider: FiatProviderName by lazy { row.provider.toPrimitives() }

    val providerName: String get() = row.providerName

    val cryptoFormatted: String by lazy { row.cryptoEstimateText(cryptoText) }

    val cryptoText: String by lazy { row.cryptoAmount.text() }

    val fiatFormatted: String by lazy { row.fiatAmount.text() }
}

fun GemFiatQuoteRow.toProviderUIModel(): BuyFiatProviderUIModel = BuyFiatProviderUIModel(row = this)
