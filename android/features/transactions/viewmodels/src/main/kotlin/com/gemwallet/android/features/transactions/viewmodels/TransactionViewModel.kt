package com.gemwallet.android.features.transactions.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.data.services.store.queries.TransactionQuery
import com.gemwallet.android.data.services.store.queries.WalletQuery
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.transactions.viewmodels.models.TransactionHeaderTarget
import com.gemwallet.android.features.transactions.viewmodels.models.target
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.requireString
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemTransactionDetailRows
import uniffi.gemstone.GemTransactionDetailSection
import uniffi.gemstone.GemTransactionDetailsServiceInterface
import uniffi.gemstone.GemTransactionsServiceInterface
import uniffi.gemstone.transactionDetailSections
import javax.inject.Inject

@HiltViewModel
class TransactionViewModel @Inject constructor(
    walletQuery: WalletQuery,
    transactionQuery: TransactionQuery,
    private val service: GemTransactionDetailsServiceInterface,
    private val transactionsService: GemTransactionsServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val walletId = WalletId(savedStateHandle.requireString(RouteArgument.WalletId))

    private val transactionId = requireNotNull(
        TransactionId.from(savedStateHandle.requireString(RouteArgument.TransactionId)),
    ) { "Invalid TransactionId route argument" }

    val data: StateFlow<GemTransactionDetailRows?> = combine(
        walletQuery(walletId).map { it?.type }.distinctUntilChanged(),
        transactionQuery(walletId, transactionId).filterNotNull(),
    ) { walletType, transaction ->
        walletType?.let { type -> service.detailRows(transaction.toGem(), type.toGem()) }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val headerTarget: StateFlow<TransactionHeaderTarget?> = data.map { it?.headerAction?.target() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val sections: StateFlow<List<GemTransactionDetailSection>> = data.map { it?.let(::transactionDetailSections).orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val _loadState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)
    val loadState: StateFlow<GemLoadState> = _loadState.asStateFlow()

    init {
        load()
    }

    fun load() {
        viewModelScope.launch(ioDispatcher) {
            _loadState.value = GemLoadState.Loading
            _loadState.value = transactionsService.loadTransaction(walletId.id, transactionId.identifier)
        }
    }
}

fun GemTransactionDetailRows.chain(): Chain = asset.toPrimitives().chain
