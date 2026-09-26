package com.gemwallet.android.features.wallets.viewmodels.models

import com.wallet.core.primitives.WalletId

data class WalletsUIState(val pinned: List<WalletItemUIModel> = emptyList(), val unpinned: List<WalletItemUIModel> = emptyList()) {
    fun name(walletId: WalletId): String = (pinned + unpinned).firstOrNull { it.row.id == walletId.id }?.row?.name.orEmpty()
}
