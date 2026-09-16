package com.gemwallet.android.domains.wallet.aggregates

import uniffi.gemstone.GemWalletRow

interface WalletDataAggregate {
    val isCurrent: Boolean
    val row: GemWalletRow
}
