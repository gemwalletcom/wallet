package com.gemwallet.android.ui.components.list_item

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.pendingColor
import uniffi.gemstone.DelegationState
import uniffi.gemstone.GemDelegationStatus
import uniffi.gemstone.GemDelegationTone

@Composable
fun GemDelegationStatus.stateText(): String = stringResource(
    when (state) {
        DelegationState.ACTIVE -> R.string.stake_active
        DelegationState.PENDING -> R.string.stake_pending
        DelegationState.INACTIVE -> R.string.stake_inactive
        DelegationState.ACTIVATING -> R.string.stake_activating
        DelegationState.DEACTIVATING -> R.string.stake_deactivating
        DelegationState.AWAITING_WITHDRAWAL -> R.string.stake_awaiting_withdrawal
    }
)

@Composable
fun GemDelegationTone.color(): Color = when (this) {
    GemDelegationTone.POSITIVE -> MaterialTheme.colorScheme.tertiary
    GemDelegationTone.PENDING -> pendingColor
    GemDelegationTone.NEGATIVE -> MaterialTheme.colorScheme.error
}
