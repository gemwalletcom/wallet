package com.gemwallet.android.testkit

import uniffi.gemstone.GemWalletPlaceholder
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.GemWalletSubtitle

fun mockGemWalletRow(id: String = "wallet-1", showsWatchBadge: Boolean = false, isPinned: Boolean = false) = GemWalletRow(
    id = id,
    name = "Wallet",
    subtitle = GemWalletSubtitle.Multicoin,
    placeholder = GemWalletPlaceholder.Multicoin,
    showsWatchBadge = showsWatchBadge,
    isPinned = isPinned,
    hasAvatar = false,
    imageUrl = null,
)
