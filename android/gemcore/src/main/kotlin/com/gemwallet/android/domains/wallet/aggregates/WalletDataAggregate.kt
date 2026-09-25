package com.gemwallet.android.domains.wallet.aggregates

import uniffi.gemstone.GemWalletRow

data class WalletDataAggregate(val row: GemWalletRow, val isCurrent: Boolean)
