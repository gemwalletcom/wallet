package com.gemwallet.android.application.wallet_connect.cases

import com.gemwallet.android.application.wallet_connect.WalletConnectSessionProposal
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemWalletConnectRejectionReason

interface ApproveWalletConnection {
    fun approveConnection(wallet: Wallet, proposal: WalletConnectSessionProposal, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit)

    fun rejectConnection(proposal: WalletConnectSessionProposal, reason: GemWalletConnectRejectionReason = GemWalletConnectRejectionReason.USER_REJECTED, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit)
}
