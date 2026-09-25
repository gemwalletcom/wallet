package com.gemwallet.android.testkit

import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import uniffi.gemstone.GemWalletPlaceholder
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.GemWalletSubtitle

fun mockWalletDataAggregate(row: GemWalletRow = mockGemWalletRow(id = "wallet-1", name = "Wallet", subtitle = GemWalletSubtitle.Multicoin, placeholder = GemWalletPlaceholder.Multicoin)): WalletDataAggregate =
    WalletDataAggregate(row = row, isCurrent = false)
