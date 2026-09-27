package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.data.services.store.queries.WalletsQuery
import com.gemwallet.android.ext.toGem
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemWalletSection
import uniffi.gemstone.walletSections

@OptIn(ExperimentalCoroutinesApi::class)
class GetAllWalletsImpl(private val getSession: GetSession, private val walletsQuery: WalletsQuery, scope: CoroutineScope = CoroutineScope(Dispatchers.IO)) : GetAllWallets {

    private val sections: StateFlow<List<GemWalletSection>> = getSession()
        .flatMapLatest { session ->
            walletsQuery().map { items -> walletSections(items.map { it.toGem() }, session?.wallet?.id?.id) }
        }
        .flowOn(Dispatchers.IO)
        .stateIn(scope, SharingStarted.Eagerly, emptyList())

    override fun getAllWallets(): StateFlow<List<GemWalletSection>> = sections
}
