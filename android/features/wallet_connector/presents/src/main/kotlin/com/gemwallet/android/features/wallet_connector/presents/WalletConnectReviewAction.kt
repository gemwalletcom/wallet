package com.gemwallet.android.features.wallet_connector.presents

import com.wallet.core.primitives.ChainAddress

internal sealed interface WalletConnectReviewAction {
    data object Approve : WalletConnectReviewAction
    data object Reject : WalletConnectReviewAction
    data class OpenAddress(val address: ChainAddress) : WalletConnectReviewAction
}
