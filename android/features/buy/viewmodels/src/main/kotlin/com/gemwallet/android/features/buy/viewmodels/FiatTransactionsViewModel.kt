package com.gemwallet.android.features.buy.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.buy.BuyRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.FiatTransactionInfo
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class FiatTransactionsViewModel @Inject constructor(
    sessionRepository: SessionRepository,
    private val buyRepository: BuyRepository,
) : ViewModel() {

    private val walletId = sessionRepository.session().map { it?.wallet?.id }

    val transactions: StateFlow<List<FiatTransactionInfo>> = walletId
        .flatMapLatest { id ->
            if (id != null) buyRepository.observeFiatTransactions(id) else flowOf(emptyList())
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    init {
        refresh()
    }

    fun refresh() {
        viewModelScope.launch(Dispatchers.IO) {
            val id = walletId.first() ?: return@launch
            buyRepository.updateFiatTransactions(id)
        }
    }
}
