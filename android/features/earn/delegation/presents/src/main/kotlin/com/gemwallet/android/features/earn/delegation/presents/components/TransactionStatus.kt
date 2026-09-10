package com.gemwallet.android.features.earn.delegation.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.stateText
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemDelegationStatus

@Composable
internal fun TransactionStatus(status: GemDelegationStatus, listPosition: ListPosition) {
    PropertyItem(
        title = R.string.transaction_status,
        data = status.stateText(),
        dataColor = status.tone.color(),
        listPosition = listPosition
    )
}
