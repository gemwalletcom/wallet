package com.gemwallet.android.features.payment.viewmodels

import com.gemwallet.android.features.payment.viewmodels.model.PaymentMerchantUIModel
import com.gemwallet.android.features.payment.viewmodels.model.PaymentOutcomeUIModel
import com.gemwallet.android.features.payment.viewmodels.model.PaymentQuoteUIModel
import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.PaymentException

sealed interface PaymentSceneState {
    data object Loading : PaymentSceneState

    data class Quotes(
        val merchant: PaymentMerchantUIModel,
        val walletName: String,
        val walletType: WalletType,
        val walletChain: Chain?,
        val price: String?,
        val quotes: List<PaymentQuoteUIModel>,
        val selected: String?,
        val collectData: String? = null,
    ) : PaymentSceneState {
        val selectedQuote: PaymentQuoteUIModel?
            get() = quotes.firstOrNull { it.id == selected }
    }

    data class Confirm(
        val params: ConfirmParams.TransferParams.Generic,
    ) : PaymentSceneState

    data class Outcome(val outcome: PaymentOutcomeUIModel) : PaymentSceneState

    data object Done : PaymentSceneState

    data class Error(val error: PaymentLinkError) : PaymentSceneState
}

sealed interface PaymentLinkError {
    data object NoWallet : PaymentLinkError
    data object WatchWallet : PaymentLinkError
    data object QuoteUnavailable : PaymentLinkError
    data object NoAccount : PaymentLinkError
    data class Gateway(val error: PaymentException?) : PaymentLinkError
}
