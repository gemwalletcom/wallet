package com.gemwallet.android.ui.components.list_item

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.pendingColor
import com.wallet.core.primitives.DelegationState
import com.wallet.core.primitives.DelegationState.Activating
import com.wallet.core.primitives.DelegationState.Active
import com.wallet.core.primitives.DelegationState.AwaitingWithdrawal
import com.wallet.core.primitives.DelegationState.Deactivating
import com.wallet.core.primitives.DelegationState.Inactive
import com.wallet.core.primitives.DelegationState.Pending

@Composable
fun DelegationState.stateText(active: Boolean): String = stringResource(
    when (this) {
        Active -> if (active) R.string.stake_active else R.string.stake_inactive
        Pending -> R.string.stake_pending
        Inactive -> R.string.stake_inactive
        Activating -> R.string.stake_activating
        Deactivating -> R.string.stake_deactivating
        AwaitingWithdrawal -> R.string.stake_awaiting_withdrawal
    }
)

@Composable
fun DelegationState.stateColor(): Color = when (this) {
    Active -> MaterialTheme.colorScheme.tertiary
    Pending,
    Activating,
    Deactivating -> pendingColor
    AwaitingWithdrawal,
    Inactive -> MaterialTheme.colorScheme.error
}
