package com.gemwallet.android.features.earn.delegation.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.stateColor
import com.gemwallet.android.ui.components.list_item.stateText
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.DelegationState

@Composable
internal fun TransactionStatus(state: DelegationState, active: Boolean, listPosition: ListPosition) {
    PropertyItem(
        title = R.string.transaction_status,
        data = state.stateText(active),
        dataColor = state.stateColor(),
        listPosition = listPosition
    )
}
