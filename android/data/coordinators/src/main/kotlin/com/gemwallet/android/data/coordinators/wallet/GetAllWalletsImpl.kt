package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.data.services.store.queries.WalletsQuery
import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import com.gemwallet.android.ext.toGem
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemWalletServiceInterface
import uniffi.gemstone.walletRows

@OptIn(ExperimentalCoroutinesApi::class)
class GetAllWalletsImpl(private val getSession: GetSession, private val walletsQuery: WalletsQuery, private val walletService: GemWalletServiceInterface, scope: CoroutineScope = CoroutineScope(Dispatchers.IO)) : GetAllWallets {

    private val wallets: StateFlow<List<WalletDataAggregate>> = walletAggregates()
        .stateIn(scope, SharingStarted.Eagerly, emptyList())

    override fun getAllWallets(): StateFlow<List<WalletDataAggregate>> = wallets

    private fun walletAggregates(): Flow<List<WalletDataAggregate>> = getSession().flatMapLatest { session ->
        val currentWalletId = session?.wallet?.id
        walletsQuery().map { items ->
            walletService.sortedWallets(items.map { it.toGem() })
        }.mapLatest { wallets ->
            walletRows(wallets).zip(wallets) { row, wallet -> WalletDataAggregate(row = row, isCurrent = wallet.id == currentWalletId?.id) }
        }
    }
        .flowOn(Dispatchers.IO)
}
