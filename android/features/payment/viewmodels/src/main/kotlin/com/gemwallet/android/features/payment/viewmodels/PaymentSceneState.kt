package com.gemwallet.android.features.payment.viewmodels

import com.gemwallet.android.features.payment.viewmodels.model.PaymentMerchantUIModel
import com.gemwallet.android.features.payment.viewmodels.model.PaymentOutcomeUIModel
import com.gemwallet.android.features.payment.viewmodels.model.PaymentQuoteUIModel
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.ui.models.PayloadField
import com.wallet.core.primitives.SimulationWarning
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
        val expiresAt: Long?,
        val expired: Boolean,
    ) : PaymentSceneState {
        val selectedQuote: PaymentQuoteUIModel?
            get() = quotes.firstOrNull { it.id == selected }
    }

    data class CollectData(val url: String) : PaymentSceneState

    data class Approve(
        val params: ConfirmParams.TokenApprovalParams,
    ) : PaymentSceneState

    data class Confirm(
        val params: ConfirmParams.TransferParams.Generic,
    ) : PaymentSceneState

    data class SignMessage(
        val merchant: PaymentMerchantUIModel,
        val chain: Chain,
        val walletName: String,
        val quote: PaymentQuoteUIModel?,
        val price: String?,
        val expiresAt: Long?,
        val plainMessage: String,
        val primaryPayloadFields: List<PayloadField>,
        val secondaryPayloadFields: List<PayloadField>,
        val warnings: List<SimulationWarning>,
        val expired: Boolean,
    ) : PaymentSceneState {
        val hasPayload: Boolean
            get() = primaryPayloadFields.isNotEmpty() || secondaryPayloadFields.isNotEmpty()
    }

    data class Outcome(val outcome: PaymentOutcomeUIModel) : PaymentSceneState

    data class Error(val error: PaymentLinkError) : PaymentSceneState
}

sealed interface PaymentLinkError {
    data object NoWallet : PaymentLinkError
    data object WatchWallet : PaymentLinkError
    data object NoQuotes : PaymentLinkError
    data object QuoteUnavailable : PaymentLinkError
    data object NoAccount : PaymentLinkError
    data object DataCollection : PaymentLinkError
    data object UnknownAsset : PaymentLinkError
    data class Gateway(val error: PaymentException?) : PaymentLinkError
}
