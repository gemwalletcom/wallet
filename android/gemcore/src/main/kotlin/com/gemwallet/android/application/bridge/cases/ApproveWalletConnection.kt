package com.gemwallet.android.application.bridge.cases

import com.gemwallet.android.application.bridge.WalletConnectSessionProposal
import com.wallet.core.primitives.Wallet

interface ApproveWalletConnection {
    fun approveConnection(wallet: Wallet, proposal: WalletConnectSessionProposal, onSuccess: () -> Unit, onError: (String) -> Unit)

    fun rejectConnection(proposal: WalletConnectSessionProposal, onSuccess: () -> Unit, onError: (String) -> Unit)
}
