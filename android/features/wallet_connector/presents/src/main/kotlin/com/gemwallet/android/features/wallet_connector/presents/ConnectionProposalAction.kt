package com.gemwallet.android.features.wallet_connector.presents

import com.wallet.core.primitives.WalletId

internal sealed interface ConnectionProposalAction {
    data object Approve : ConnectionProposalAction
    data object Reject : ConnectionProposalAction
    data class SelectWallet(val walletId: WalletId) : ConnectionProposalAction
}
