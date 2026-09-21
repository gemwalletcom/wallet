package com.gemwallet.android.testkit

import com.gemwallet.android.application.wallet_connect.values.WalletConnectPairingProposal
import com.wallet.core.primitives.WalletConnectionSessionProposal
import uniffi.gemstone.WalletConnectionVerificationStatus

fun mockWalletConnectPairingProposal(proposal: WalletConnectionSessionProposal = mockWalletConnectionSessionProposal()) = WalletConnectPairingProposal(
    proposal = proposal,
    verificationStatus = WalletConnectionVerificationStatus.VERIFIED,
)
