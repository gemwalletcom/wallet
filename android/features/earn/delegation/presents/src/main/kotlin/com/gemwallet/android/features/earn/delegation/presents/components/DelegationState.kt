package com.gemwallet.android.features.earn.delegation.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemDelegationCompletion

@Composable
internal fun DelegationState(completion: GemDelegationCompletion, availableIn: String, listPosition: ListPosition) {
    PropertyItem(
        title = when (completion) {
            GemDelegationCompletion.ACTIVE_IN -> R.string.stake_active_in
            GemDelegationCompletion.AVAILABLE_IN -> R.string.stake_available_in
        },
        data = availableIn,
        listPosition = listPosition,
    )
}
