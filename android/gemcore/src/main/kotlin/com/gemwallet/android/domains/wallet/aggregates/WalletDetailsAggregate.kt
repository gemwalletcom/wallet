package com.gemwallet.android.domains.wallet.aggregates

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType

interface WalletDetailsAggregate {
    val id: WalletId
    val name: String
    val type: WalletType
    val walletChain: Chain?
    val accounts: List<ChainAddress>
    val imageUrl: String?

    val hasAvatar: Boolean
        get() = !imageUrl.isNullOrEmpty()
}
