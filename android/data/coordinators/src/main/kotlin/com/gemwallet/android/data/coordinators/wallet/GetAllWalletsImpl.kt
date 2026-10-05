package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.data.services.store.queries.WalletListItemsQuery
import com.gemwallet.android.ext.toGem
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemWalletSection
import uniffi.gemstone.walletSections

class GetAllWalletsImpl(getSession: GetSession, walletListItemsQuery: WalletListItemsQuery, scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)) : GetAllWallets {

    private val currentWalletId = getSession().map { it?.wallet?.id?.id }.distinctUntilChanged()

    private val sections: StateFlow<List<GemWalletSection>> = combine(walletListItemsQuery(), currentWalletId) { items, walletId ->
        walletSections(items.map { it.toGem() }, walletId)
    }
        .flowOn(Dispatchers.IO)
        .stateIn(scope, SharingStarted.Eagerly, emptyList())

    override fun getAllWallets(): StateFlow<List<GemWalletSection>> = sections
}
