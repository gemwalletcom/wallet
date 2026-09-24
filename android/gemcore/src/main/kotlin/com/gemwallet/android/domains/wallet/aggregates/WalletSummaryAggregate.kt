package com.gemwallet.android.domains.wallet.aggregates

import com.gemwallet.android.model.text
import uniffi.gemstone.GemBannerRow
import uniffi.gemstone.GemHeaderActions
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.GemWalletHomeViewState
import uniffi.gemstone.GemWalletRow

interface WalletSummaryAggregate {
    val state: GemWalletHomeViewState
    val walletRow: GemWalletRow
    val isBalanceHidden: Boolean

    val walletTotalValue: String get() = state.total.text()
    val changedValue: GemLocalizedText? get() = state.pnl
    val changeTone: GemValueTone get() = state.pnlTone
    val headerActions: GemHeaderActions get() = state.headerActions
    val showCollections: Boolean get() = state.showCollections
    val showPerpetuals: Boolean get() = state.showsPerpetuals
    val banners: List<GemBannerRow> get() = state.visibleBanners
}
