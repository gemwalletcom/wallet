package com.gemwallet.android.features.wallets.viewmodels.models

import com.gemwallet.android.ui.style.iconModel
import com.gemwallet.android.ui.style.supportIcon
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemAddressRow
import uniffi.gemstone.GemWalletDetails

data class WalletDetailUIModel(val walletId: WalletId, val name: String, val avatar: WalletAvatarUIModel, val address: GemAddressRow?)

internal fun GemWalletDetails.uiModel() = WalletDetailUIModel(
    walletId = WalletId(row.id),
    name = row.name,
    avatar = WalletAvatarUIModel(
        imageUrl = row.imageUrl,
        placeholder = row.placeholder.iconModel(),
        supportIcon = row.supportIcon(),
        canRemove = row.hasAvatar,
    ),
    address = address,
)
