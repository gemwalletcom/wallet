package com.gemwallet.android.features.wallet.viewmodels.models

import com.gemwallet.android.ui.components.list_item.ListItemModel
import uniffi.gemstone.GemWalletSecretKind

data class WalletSecretUIModel(
    val secretKind: GemWalletSecretKind,
    val model: ListItemModel,
)
