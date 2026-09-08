package com.gemwallet.android.application.wallet.cases

import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.mapLatest

interface GetAllWallets {
    fun getAllWallets(): StateFlow<List<WalletDataAggregate>>
}

@OptIn(ExperimentalCoroutinesApi::class)
fun Flow<List<WalletDataAggregate>>.pinned(): Flow<List<WalletDataAggregate>> = this.mapLatest { items -> items.filter { it.isPinned } }

@OptIn(ExperimentalCoroutinesApi::class)
fun Flow<List<WalletDataAggregate>>.unpinned(): Flow<List<WalletDataAggregate>> = this.mapLatest { items -> items.filter { !it.isPinned } }