package com.gemwallet.android.domains.wallet.aggregates

import com.gemwallet.android.domains.banner.BannerRow
import com.gemwallet.android.domains.price.values.EquivalentValue
import uniffi.gemstone.GemHeaderActions
import uniffi.gemstone.GemWalletRow

interface WalletSummaryAggregate {
    val walletRow: GemWalletRow
    val walletTotalValue: String
    val changedValue: EquivalentValue?
    val isBalanceHidden: Boolean
    val headerActions: GemHeaderActions
    val showCollections: Boolean
    val banners: List<BannerRow>
}
