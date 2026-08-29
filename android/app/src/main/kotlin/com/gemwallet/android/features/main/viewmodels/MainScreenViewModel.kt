package com.gemwallet.android.features.main.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.PendingNavigationCoordinator
import com.gemwallet.android.application.transactions.cases.GetPendingTransactionsCount
import com.gemwallet.android.application.wallet_connect.cases.IsWalletConnectEnabled
import com.gemwallet.android.application.session.cases.GetSession
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class MainScreenViewModel @Inject constructor(
    private val getSession: GetSession,
    private val pendingNavigationCoordinator: PendingNavigationCoordinator,
    isWalletConnectEnabledCase: IsWalletConnectEnabled,
    getTransactions: GetPendingTransactionsCount
) : ViewModel() {
    val isWalletConnectEnabled: Boolean = isWalletConnectEnabledCase.isWalletConnectEnabled()

    val pendingTxCount = getSession()
        .filterNotNull()
        .flatMapLatest { getTransactions.getPendingTransactionsCount() }
        .filterNotNull()
        .map { if (it == 0) null else it.toString() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun onScan(code: String) = pendingNavigationCoordinator.handleScan(code)
}
