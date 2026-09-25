package com.gemwallet.android.testkit

import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import uniffi.gemstone.GemWalletRow

fun mockWalletDataAggregate(row: GemWalletRow = mockGemWalletRow()): WalletDataAggregate = WalletDataAggregate(row = row, isCurrent = false)
