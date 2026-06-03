package com.gemwallet.android.data.coordinators.wallet

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.wallet.coordinators.GetWalletDetails
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.domains.wallet.aggregates.WalletDetailsAggregate
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetWalletDetailsImpl(
    private val walletsRepository: WalletsRepository
) : GetWalletDetails {

    override fun getWallet(walletId: WalletId): Flow<WalletDetailsAggregate?> {
        return  walletsRepository.getWallet(walletId)
            .mapLatest { dto -> dto?.let { WalletDetailsAggregateImpl(it) } }
    }
}

@Stable
class WalletDetailsAggregateImpl(wallet: Wallet) : WalletDetailsAggregate {
    override val id: WalletId = wallet.id
    override val name: String = wallet.name
    override val type: WalletType = wallet.type
    override val walletChain: Chain? = wallet.accounts.firstOrNull()?.chain
    override val addresses: List<String> = wallet.accounts.map { it.address }
    override val imageUrl: String? = wallet.imageUrl
}
