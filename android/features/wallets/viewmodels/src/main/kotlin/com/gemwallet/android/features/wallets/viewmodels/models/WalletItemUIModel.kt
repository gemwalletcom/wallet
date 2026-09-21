package com.gemwallet.android.features.wallets.viewmodels.models

import com.gemwallet.android.ui.components.list_item.WalletRowUIModel
import com.wallet.core.primitives.WalletId

data class WalletItemUIModel(val walletId: WalletId, val row: WalletRowUIModel, val isCurrent: Boolean, val isPinned: Boolean)

data class WalletsUIState(val pinned: List<WalletItemUIModel> = emptyList(), val unpinned: List<WalletItemUIModel> = emptyList()) {
    fun name(walletId: WalletId): String = (pinned + unpinned).firstOrNull { it.walletId == walletId }?.row?.name.orEmpty()
}
