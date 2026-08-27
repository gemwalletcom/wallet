package com.gemwallet.android.application.wallet_connect.coordinators

import com.gemwallet.android.application.wallet_connect.values.WalletConnectPairingProposal
import uniffi.gemstone.WalletConnectionVerificationStatus

interface PrepareSessionProposal {
    suspend operator fun invoke(
        name: String,
        description: String,
        url: String,
        icons: List<String>,
        requiredChainIds: List<String>,
        optionalChainIds: List<String>,
        origin: String?,
        validation: WalletConnectionVerificationStatus,
    ): WalletConnectPairingProposal
}
