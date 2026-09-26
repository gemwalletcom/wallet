package com.gemwallet.android.features.fiat_connect.viewmodels.models

import androidx.compose.runtime.Stable
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.wallet.core.primitives.FiatProviderName
import uniffi.gemstone.GemFiatQuoteRow

@Stable
data class FiatQuoteUIModel(val row: GemFiatQuoteRow) {

    val provider: FiatProviderName by lazy { row.provider.toPrimitives() }

    val providerName: String get() = row.providerName

    val cryptoFormatted: String by lazy { row.cryptoEstimateText(cryptoText) }

    val cryptoText: String by lazy { row.cryptoAmount.text() }

    val fiatFormatted: String by lazy { row.fiatAmount.text() }
}

fun GemFiatQuoteRow.toQuoteUIModel(): FiatQuoteUIModel = FiatQuoteUIModel(row = this)
