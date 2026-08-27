package com.gemwallet.android.application.wallet_connect.values

import com.wallet.core.primitives.WalletConnectionSessionProposal
import uniffi.gemstone.WalletConnectionVerificationStatus

data class WalletConnectPairingProposal(
    val proposal: WalletConnectionSessionProposal,
    val verificationStatus: WalletConnectionVerificationStatus,
)
