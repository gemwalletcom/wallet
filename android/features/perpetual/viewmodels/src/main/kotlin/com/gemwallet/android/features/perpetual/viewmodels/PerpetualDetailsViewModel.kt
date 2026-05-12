package com.gemwallet.android.features.perpetual.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetual
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualChartData
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualPosition
import com.gemwallet.android.application.transactions.coordinators.GetTransactions
import com.gemwallet.android.application.transactions.coordinators.SyncAssetTransactions
import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.TransactionType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PerpetualDetailsViewModel @Inject constructor(
    private val getPerpetual: GetPerpetual,
    private val getPerpetualPosition: GetPerpetualPosition,
    private val getPerpetualChartData: GetPerpetualChartData,
    private val getTransactions: GetTransactions,
    private val syncAssetTransactions: SyncAssetTransactions,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val assetId = savedStateHandle.requireAssetId()

    private val transactionFilters = listOf(
        TransactionsRequestFilter.Asset(assetId),
        TransactionsRequestFilter.Types(
            listOf(
                TransactionType.PerpetualOpenPosition,
                TransactionType.PerpetualClosePosition,
            )
        )
    )

    private val transactionSync = flow {
        syncAssetTransactions.syncAssetTransactions(assetId)
        emit(Unit)
    }
        .onStart { emit(Unit) }
        .flowOn(Dispatchers.IO)

    val perpetual = getPerpetual.getPerpetualByAssetId(assetId)
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val position = perpetual
        .flatMapLatest { perpetual ->
            perpetual?.let { getPerpetualPosition.getPositionByPerpetual(it.id) } ?: flowOf(null)
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val transactions = combine(
        getTransactions.getTransactions(transactionFilters),
        transactionSync,
    ) { transactions, _ -> transactions }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val period = MutableStateFlow(ChartPeriod.Day)
    val chart = period
        .mapLatest { period -> getPerpetualChartData.getPerpetualChartData(assetId, period) }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun period(period: ChartPeriod) {
        this.period.update { period }
    }
}
