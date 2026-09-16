package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import androidx.compose.runtime.Stable
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.walletRows
import uniffi.gemstone.GemWalletService
import uniffi.gemstone.GemWalletServiceInterface
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.flowOn

@OptIn(ExperimentalCoroutinesApi::class)
class GetAllWalletsImpl(
    private val getSession: GetSession,
    private val walletStore: GemstoneWalletStore,
    private val walletService: GemWalletServiceInterface,
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetAllWallets {

    private val wallets: StateFlow<List<WalletDataAggregate>> = walletAggregates()
        .stateIn(scope, SharingStarted.Eagerly, emptyList())

    override fun getAllWallets(): StateFlow<List<WalletDataAggregate>> = wallets

    private fun walletAggregates(): Flow<List<WalletDataAggregate>> {
        return getSession().flatMapLatest { session ->
            val currentWalletId = session?.wallet?.id
            walletStore.observeWallets().map { items ->
                walletService.sortedWallets(items.map { it.toGem() }).map { it.toPrimitives() }
            }.mapLatest { items ->
                val rows = walletRows(items.map { it.toGem() })
                items.mapIndexed { index, wallet ->
                    WalletDataAggregateImpl(
                        row = rows[index],
                        isCurrent = wallet.id == currentWalletId,
                    )
                }
            }
        }
        .flowOn(Dispatchers.IO)
    }
}

@Stable
class WalletDataAggregateImpl(
    override val row: GemWalletRow,
    override val isCurrent: Boolean,
) : WalletDataAggregate
