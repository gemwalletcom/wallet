package com.gemwallet.android.features.activities.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.transactions.coordinators.GetTransactionDetails
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.TransactionId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@HiltViewModel
class TransactionDetailsViewModel @Inject constructor(
    private val getTransactionDetails: GetTransactionDetails,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val transactionId = requireNotNull(
        TransactionId.from(savedStateHandle.requireString(RouteArgument.TransactionId))
    ) { "Invalid TransactionId route argument" }

    val data = getTransactionDetails.getTransactionDetails(transactionId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
}

private fun SavedStateHandle.requireString(argument: RouteArgument): String {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    check(value.isNotBlank()) { "Blank route argument: ${argument.key}" }
    return value
}
