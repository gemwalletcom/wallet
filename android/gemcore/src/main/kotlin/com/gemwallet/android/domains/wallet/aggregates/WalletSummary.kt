package com.gemwallet.android.domains.wallet.aggregates

import uniffi.gemstone.GemWalletHomeViewState
import uniffi.gemstone.GemWalletRow

data class WalletSummary(val state: GemWalletHomeViewState, val walletRow: GemWalletRow, val isBalanceHidden: Boolean)
