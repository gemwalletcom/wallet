package com.gemwallet.android.domains.wallet.aggregates

import uniffi.gemstone.GemBannerRow
import uniffi.gemstone.GemHeaderActions
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.GemWalletRow

interface WalletSummaryAggregate {
    val walletRow: GemWalletRow
    val walletTotalValue: String
    val changedValue: GemLocalizedText?
    val changeTone: GemValueTone
    val isBalanceHidden: Boolean
    val headerActions: GemHeaderActions
    val showCollections: Boolean
    val banners: List<GemBannerRow>
}
