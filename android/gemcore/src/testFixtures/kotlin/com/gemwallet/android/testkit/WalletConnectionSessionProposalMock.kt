package com.gemwallet.android.testkit

import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnectionSessionProposal

fun mockWalletConnectionSessionProposal(
    defaultWallet: Wallet = mockWallet(),
    wallets: List<Wallet> = listOf(defaultWallet),
) = WalletConnectionSessionProposal(
    defaultWallet = defaultWallet,
    wallets = wallets,
    metadata = mockApplicationMetadata(),
)
