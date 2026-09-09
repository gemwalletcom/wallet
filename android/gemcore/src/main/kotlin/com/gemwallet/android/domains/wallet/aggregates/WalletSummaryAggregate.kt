package com.gemwallet.android.domains.wallet.aggregates

import com.gemwallet.android.domains.price.values.EquivalentValue
import uniffi.gemstone.GemHeaderActions

interface WalletSummaryAggregate {
    val walletName: String
    val walletIcon: WalletIcon
    val walletTotalValue: String
    val changedValue: EquivalentValue?
    val isBalanceHidden: Boolean
    val headerActions: GemHeaderActions
}
