package com.gemwallet.android.features.main.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.PendingNavigationCoordinator
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.TransactionsCountQuery
import com.gemwallet.android.ext.toPrimitives
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.pendingActivityFilters
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class MainScreenViewModel @Inject constructor(getCurrentWalletId: GetCurrentWalletId, private val pendingNavigationCoordinator: PendingNavigationCoordinator, transactionsCountQuery: TransactionsCountQuery) : ViewModel() {
    val pendingTxCount: StateFlow<Int> = getCurrentWalletId()
        .flatMapLatest { walletId -> transactionsCountQuery(walletId, pendingActivityFilters().toPrimitives()) }
        .filterNotNull()
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    fun onScan(code: String) = pendingNavigationCoordinator.pendScan(code)
}
