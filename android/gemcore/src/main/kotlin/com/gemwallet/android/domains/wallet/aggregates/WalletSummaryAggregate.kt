package com.gemwallet.android.domains.wallet.aggregates

import com.gemwallet.android.domains.price.values.EquivalentValue
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemHeaderButton

interface WalletSummaryAggregate {
    val walletType: WalletType
    val walletName: String
    val walletIcon: WalletIcon
    val walletTotalValue: String
    val changedValue: EquivalentValue?
    val isBalanceHidden: Boolean
    val headerButtons: List<GemHeaderButton>
}