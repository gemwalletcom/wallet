package com.gemwallet.android.features.activities.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.transactions.cases.GetTransactionDetails
import com.gemwallet.android.features.activities.viewmodels.models.TransactionDetailsRowUIModel
import com.gemwallet.android.features.activities.viewmodels.models.TransactionHeaderTarget
import com.gemwallet.android.features.activities.viewmodels.models.target
import com.gemwallet.android.features.activities.viewmodels.models.uiModel
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.TransactionId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

@HiltViewModel
class TransactionDetailsViewModel @Inject constructor(
    private val getTransactionDetails: GetTransactionDetails,
    savedStateHandle: SavedStateHandle,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val transactionId = requireNotNull(
        TransactionId.from(savedStateHandle.requireString(RouteArgument.TransactionId))
    ) { "Invalid TransactionId route argument" }

    val data = getTransactionDetails.getTransactionDetails(transactionId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val headerTarget: StateFlow<TransactionHeaderTarget?> = data.map { it?.headerAction?.target() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val sections: StateFlow<List<ListSection<TransactionDetailsRowUIModel>>> = data.map { details ->
        details?.sections.orEmpty().mapIndexed { index, section ->
            ListSection(id = index.toString(), items = section.rows.map { row -> details!!.value(row).uiModel(context, details.asset) })
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
}

private fun SavedStateHandle.requireString(argument: RouteArgument): String {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    check(value.isNotBlank()) { "Blank route argument: ${argument.key}" }
    return value
}
