package com.gemwallet.android.features.earn.delegation.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.earn.delegation.presents.localization.stringRes
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemDelegationCompletion

@Composable
internal fun DelegationState(completion: GemDelegationCompletion, availableIn: String, listPosition: ListPosition) {
    PropertyItem(
        title = completion.stringRes(),
        data = availableIn,
        listPosition = listPosition,
    )
}
