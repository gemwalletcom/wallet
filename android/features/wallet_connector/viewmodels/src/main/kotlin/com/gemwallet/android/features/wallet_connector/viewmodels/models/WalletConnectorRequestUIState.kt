package com.gemwallet.android.features.wallet_connector.viewmodels.models

import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import uniffi.gemstone.SimulationResult

sealed interface WalletConnectorRequestUIState {
    data object Loading : WalletConnectorRequestUIState

    data class SignMessage(val request: WalletConnectPendingRequest.SignMessage) : WalletConnectorRequestUIState

    data class Transaction(val input: ConfirmTransferInput, val simulation: SimulationResult) : WalletConnectorRequestUIState
}
