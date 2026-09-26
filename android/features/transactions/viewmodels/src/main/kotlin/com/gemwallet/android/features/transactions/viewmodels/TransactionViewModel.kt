package com.gemwallet.android.features.transactions.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.data.services.store.queries.TransactionQuery
import com.gemwallet.android.data.services.store.queries.WalletQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.transactions.viewmodels.models.TransactionHeaderTarget
import com.gemwallet.android.features.transactions.viewmodels.models.TransactionItemUIModel
import com.gemwallet.android.features.transactions.viewmodels.models.target
import com.gemwallet.android.features.transactions.viewmodels.models.uiModel
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemTransactionDetailRows
import uniffi.gemstone.GemTransactionDetailsServiceInterface
import uniffi.gemstone.transactionDetailSections
import javax.inject.Inject

@HiltViewModel
class TransactionViewModel @Inject constructor(
    walletQuery: WalletQuery,
    transactionQuery: TransactionQuery,
    private val service: GemTransactionDetailsServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val walletId = WalletId(savedStateHandle.requireString(RouteArgument.WalletId))

    private val transactionId = requireNotNull(
        TransactionId.from(savedStateHandle.requireString(RouteArgument.TransactionId)),
    ) { "Invalid TransactionId route argument" }

    val data: StateFlow<GemTransactionDetailRows?> = combine(
        walletQuery(walletId).map { it?.type }.distinctUntilChanged(),
        transactionQuery(walletId, transactionId),
    ) { walletType, transaction ->
        walletType?.let { type -> transaction?.let { service.detailRows(it.toGem(), type.toGem()) } }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val headerTarget: StateFlow<TransactionHeaderTarget?> = data.map { it?.headerAction?.target() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val sections: StateFlow<List<ListSection<TransactionItemUIModel>>> = data.map { details ->
        details?.let { rows ->
            transactionDetailSections(rows).mapIndexed { index, section ->
                ListSection(id = index.toString(), items = section.rows.map { row -> rows.uiModel(row, context) })
            }
        }.orEmpty()
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
}

private fun SavedStateHandle.requireString(argument: RouteArgument): String {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    check(value.isNotBlank()) { "Blank route argument: ${argument.key}" }
    return value
}
