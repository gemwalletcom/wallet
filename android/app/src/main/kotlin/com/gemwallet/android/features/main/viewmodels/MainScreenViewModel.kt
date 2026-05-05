package com.gemwallet.android.features.main.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.transactions.coordinators.GetPendingTransactionsCount
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.isNftSupported
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class MainScreenViewModel @Inject constructor(
    private val sessionRepository: SessionRepository,
    getTransactions: GetPendingTransactionsCount
) : ViewModel() {
    val pendingTxCount = sessionRepository.session()
        .filterNotNull()
        .flatMapLatest { getTransactions.getPendingTransactionsCount() }
        .filterNotNull()
        .map { if (it == 0) null else it.toString() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val collectionsAvailable = sessionRepository.session()
        .mapLatest { session -> session?.wallet?.isCollectionsAvailable() ?: false }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)
}

private fun Wallet.isCollectionsAvailable(): Boolean {
    return when (type) {
        WalletType.Multicoin -> true
        WalletType.Single,
        WalletType.PrivateKey,
        WalletType.View -> accounts.firstOrNull()?.chain?.isNftSupported() ?: false
    }
}
