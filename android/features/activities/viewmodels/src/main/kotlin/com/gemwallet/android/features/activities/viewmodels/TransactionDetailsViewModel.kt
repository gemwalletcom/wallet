package com.gemwallet.android.features.activities.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.GetHideBalancesState
import com.gemwallet.android.application.transactions.coordinators.GetTransactionDetails
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.TransactionId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class TransactionDetailsViewModel @Inject constructor(
    private val getTransactionDetails: GetTransactionDetails,
    getHideBalancesState: GetHideBalancesState,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val transactionId = requireNotNull(
        TransactionId.from(savedStateHandle.requireString(RouteArgument.TransactionId))
    ) { "Invalid TransactionId route argument" }

    val hideBalance = getHideBalancesState()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val data = hideBalance
        .flatMapLatest { hide -> getTransactionDetails.getTransactionDetails(transactionId, hide) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
}

private fun SavedStateHandle.requireString(argument: RouteArgument): String {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    check(value.isNotBlank()) { "Blank route argument: ${argument.key}" }
    return value
}
