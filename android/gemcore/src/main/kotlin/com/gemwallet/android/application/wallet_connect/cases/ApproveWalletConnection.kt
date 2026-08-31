package com.gemwallet.android.application.wallet_connect.cases

import com.gemwallet.android.application.wallet_connect.WalletConnectSessionProposal
import com.wallet.core.primitives.Wallet

interface ApproveWalletConnection {
    fun approveConnection(wallet: Wallet, proposal: WalletConnectSessionProposal, onSuccess: () -> Unit, onError: (String) -> Unit)

    fun rejectConnection(proposal: WalletConnectSessionProposal, onSuccess: () -> Unit, onError: (String) -> Unit)
}
