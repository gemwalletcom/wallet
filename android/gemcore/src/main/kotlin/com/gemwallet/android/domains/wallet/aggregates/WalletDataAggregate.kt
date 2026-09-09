package com.gemwallet.android.domains.wallet.aggregates

import uniffi.gemstone.GemWalletRow

interface WalletDataAggregate {
    val id: String
    val isCurrent: Boolean
    val name: String
    val row: GemWalletRow
    val isPinned: Boolean
    val imageUrl: String?
}
