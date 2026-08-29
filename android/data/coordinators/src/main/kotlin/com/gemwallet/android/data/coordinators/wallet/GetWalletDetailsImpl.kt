package com.gemwallet.android.data.coordinators.wallet

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.wallet.cases.GetWalletDetails
import com.gemwallet.android.data.adapters.gemstone.GemstoneWalletStore
import com.gemwallet.android.domains.wallet.aggregates.WalletDetailsAggregate
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetWalletDetailsImpl(
    private val walletStore: GemstoneWalletStore
) : GetWalletDetails {

    override fun getWallet(walletId: WalletId): Flow<WalletDetailsAggregate?> {
        return  walletStore.observeWallet(walletId)
            .mapLatest { dto -> dto?.let { WalletDetailsAggregateImpl(it) } }
    }
}

@Stable
class WalletDetailsAggregateImpl(wallet: Wallet) : WalletDetailsAggregate {
    override val id: WalletId = wallet.id
    override val name: String = wallet.name
    override val type: WalletType = wallet.type
    override val walletChain: Chain? = wallet.accounts.firstOrNull()?.chain
    override val accounts: List<ChainAddress> = wallet.accounts.map {
        ChainAddress(chain = it.chain, address = it.address)
    }
    override val imageUrl: String? = wallet.imageUrl
}
