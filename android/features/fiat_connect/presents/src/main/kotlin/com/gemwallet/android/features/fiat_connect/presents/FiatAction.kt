package com.gemwallet.android.features.fiat_connect.presents

import com.wallet.core.primitives.FiatProviderName
import uniffi.gemstone.GemFiatSuggestedAmount

internal sealed interface FiatAction {
    data class SetAmount(val amount: String) : FiatAction
    data class SelectAmount(val amount: GemFiatSuggestedAmount) : FiatAction
    data class SelectProvider(val provider: FiatProviderName) : FiatAction
    data object RetryQuote : FiatAction
    data object Continue : FiatAction
    data object OpenTransactions : FiatAction
    data object Cancel : FiatAction
}
