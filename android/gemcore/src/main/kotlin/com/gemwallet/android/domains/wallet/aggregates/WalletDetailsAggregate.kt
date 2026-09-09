package com.gemwallet.android.domains.wallet.aggregates

import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.GemWalletSecretKind

interface WalletDetailsAggregate {
    val id: WalletId
    val name: String
    val secretKind: GemWalletSecretKind?
    val row: GemWalletRow
    val accounts: List<ChainAddress>
    val imageUrl: String?

    val hasAvatar: Boolean
        get() = !imageUrl.isNullOrEmpty()
}
