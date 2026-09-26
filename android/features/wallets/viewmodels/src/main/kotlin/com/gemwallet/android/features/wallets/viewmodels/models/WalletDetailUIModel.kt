package com.gemwallet.android.features.wallets.viewmodels.models

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.components.list_item.iconModel
import com.gemwallet.android.ui.components.list_item.supportIcon
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemWalletDetails

data class WalletDetailUIModel(val walletId: WalletId, val name: String, val avatar: WalletAvatarUIModel, val address: ChainAddress?, val addressExplorer: BlockExplorerLink?)

internal fun GemWalletDetails.uiModel() = WalletDetailUIModel(
    walletId = WalletId(row.id),
    name = row.name,
    avatar = WalletAvatarUIModel(
        imageUrl = row.imageUrl,
        placeholder = row.placeholder.iconModel(),
        supportIcon = row.supportIcon(),
        canRemove = row.hasAvatar,
    ),
    address = address?.toPrimitives(),
    addressExplorer = addressExplorer?.toPrimitives(),
)
