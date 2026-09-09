package com.gemwallet.android.domains.wallet.aggregates

import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemWalletRow

interface WalletDetailsAggregate {
    val id: WalletId
    val name: String
    val type: WalletType
    val row: GemWalletRow
    val accounts: List<ChainAddress>
    val imageUrl: String?

    val hasAvatar: Boolean
        get() = !imageUrl.isNullOrEmpty()
}
