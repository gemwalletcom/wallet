package com.gemwallet.android.features.wallet.viewmodels.models

import com.gemwallet.android.domains.wallet.aggregates.WalletDetailsAggregate
import com.gemwallet.android.ui.components.list_item.iconModel
import com.gemwallet.android.ui.components.list_item.supportIcon
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.WalletId

data class WalletAvatarUIModel(val imageUrl: String?, val placeholder: Any?, val supportIcon: String?, val canRemove: Boolean)

data class WalletDetailsUIModel(val walletId: WalletId, val name: String, val avatar: WalletAvatarUIModel, val address: ChainAddress?, val addressExplorer: BlockExplorerLink?)

internal fun WalletDetailsAggregate.uiModel() = WalletDetailsUIModel(
    walletId = id,
    name = row.name,
    avatar = WalletAvatarUIModel(
        imageUrl = row.imageUrl,
        placeholder = row.placeholder.iconModel(),
        supportIcon = row.supportIcon(),
        canRemove = row.hasAvatar,
    ),
    address = address,
    addressExplorer = addressExplorer,
)
