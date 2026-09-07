package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import androidx.compose.runtime.Stable
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemWalletService
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.flowOn

@OptIn(ExperimentalCoroutinesApi::class)
class GetAllWalletsImpl(
    private val getSession: GetSession,
    private val walletStore: GemstoneWalletStore,
    private val walletService: GemWalletService,
) : GetAllWallets {

    override fun getAllWallets(): Flow<List<WalletDataAggregate>> {
        return getSession().flatMapLatest { session ->
            val currentWalletId = session?.wallet?.id
            walletStore.observeWallets().map { items ->
                walletService.sortedWallets(items.map { it.toGem() }).map { it.toPrimitives() }
            }.mapLatest { items ->
                items.map {
                    WalletDataAggregateImpl(
                        wallet = it,
                        isCurrent = it.id == currentWalletId,
                        walletAccount = it.accounts.firstOrNull(),
                    )
                }
            }
        }
        .flowOn(Dispatchers.IO)
    }
}

@Stable
class WalletDataAggregateImpl(
    private val wallet: Wallet,
    override val isCurrent: Boolean,
    private val walletAccount: Account?,
) : WalletDataAggregate {

    override val id: String = wallet.id.id

    override val name: String = wallet.name

    override val type: WalletType = wallet.type

    override val walletChain: Chain? = walletAccount?.chain

    override val walletAddress: String? = walletAccount?.address

    override val isPinned: Boolean = wallet.isPinned

    override val imageUrl: String? = wallet.imageUrl
}
