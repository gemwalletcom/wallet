package com.gemwallet.android.features.swap.views.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.theme.mainActionHeight
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.features.swap.viewmodels.models.SwapActionState
import com.gemwallet.android.features.swap.viewmodels.models.SwapError
import com.gemwallet.android.features.swap.viewmodels.models.SwapUiState

@Composable
internal fun SwapAction(
    swapState: SwapUiState,
    onSwap: () -> Unit,
) {
    val action = swapState.action
    Button(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(mainActionHeight),
        onClick = onSwap,
        enabled = when (action) {
            SwapActionState.Ready,
            is SwapActionState.TransferError -> true
            is SwapActionState.QuoteError -> action.error !is SwapError.InsufficientBalance
            SwapActionState.None,
            SwapActionState.QuoteLoading,
            SwapActionState.TransferLoading -> false
        }
    ) {
        when (action) {
            SwapActionState.None,
            SwapActionState.Ready -> Text(
                text = stringResource(R.string.wallet_swap),
                style = MaterialTheme.typography.bodyLarge,
            )
            SwapActionState.QuoteLoading -> Text(
                text = stringResource(R.string.wallet_swap),
                style = MaterialTheme.typography.bodyLarge,
            )
            SwapActionState.TransferLoading -> CircularProgressIndicator20(color = Color.White)
            is SwapActionState.QuoteError,
            is SwapActionState.TransferError -> Text(
                modifier = Modifier.padding(paddingHalfSmall),
                text = when (val error = swapState.error) {
                    is SwapError.InsufficientBalance -> stringResource(R.string.transfer_insufficient_balance, error.symbol)
                    is SwapError.InputAmountTooSmall -> stringResource(R.string.stake_minimum_amount)
                    else -> stringResource(R.string.common_try_again)
                },
                style = MaterialTheme.typography.bodyLarge,
            )
        }
    }
}
