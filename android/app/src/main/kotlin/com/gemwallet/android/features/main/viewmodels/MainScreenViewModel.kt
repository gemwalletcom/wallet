package com.gemwallet.android.features.main.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.PendingNavigationCoordinator
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.TransactionsCountQuery
import com.gemwallet.android.ext.toPrimitives
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.pendingActivityFilters
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class MainScreenViewModel @Inject constructor(private val getSession: GetSession, private val pendingNavigationCoordinator: PendingNavigationCoordinator, transactionsCountQuery: TransactionsCountQuery) : ViewModel() {
    val pendingTxCount = getSession()
        .filterNotNull()
        .flatMapLatest { session -> transactionsCountQuery(session.wallet.id, pendingActivityFilters().toPrimitives()) }
        .filterNotNull()
        .map { if (it == 0) null else it.toString() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun onScan(code: String) = pendingNavigationCoordinator.pendScan(code)
}
