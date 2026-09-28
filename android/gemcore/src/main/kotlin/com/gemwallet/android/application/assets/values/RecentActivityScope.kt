package com.gemwallet.android.application.assets.values

import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemRecentActivityScope

sealed interface RecentActivityScope {
    data class Wallet(val walletId: WalletId) : RecentActivityScope
    data object AllWallets : RecentActivityScope
}

fun GemRecentActivityScope.toScope(): RecentActivityScope = when (this) {
    is GemRecentActivityScope.Wallet -> RecentActivityScope.Wallet(WalletId(walletId))
    GemRecentActivityScope.AllWallets -> RecentActivityScope.AllWallets
}
