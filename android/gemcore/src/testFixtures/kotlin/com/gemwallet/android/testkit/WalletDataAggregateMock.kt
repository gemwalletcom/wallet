package com.gemwallet.android.testkit

import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import uniffi.gemstone.GemWalletRow

fun mockWalletDataAggregate(row: GemWalletRow = mockGemWalletRow()): WalletDataAggregate = object : WalletDataAggregate {
    override val isCurrent: Boolean = false
    override val row: GemWalletRow = row
}
