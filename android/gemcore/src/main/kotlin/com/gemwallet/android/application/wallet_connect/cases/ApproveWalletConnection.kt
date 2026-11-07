package com.gemwallet.android.application.wallet_connect.cases

import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemWalletConnectProposal
import uniffi.gemstone.GemWalletConnectRejectionReason

interface ApproveWalletConnection {
    fun approveConnection(wallet: Wallet, proposal: GemWalletConnectProposal, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit)

    fun rejectConnection(proposal: GemWalletConnectProposal, reason: GemWalletConnectRejectionReason = GemWalletConnectRejectionReason.USER_REJECTED, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit)
}
