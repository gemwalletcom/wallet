package com.gemwallet.android.domains.wallet.aggregates

import uniffi.gemstone.GemWalletPlaceholder

data class WalletIcon(
    val imageUrl: String?,
    val placeholder: GemWalletPlaceholder,
)
