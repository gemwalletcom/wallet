package com.gemwallet.android.features.swap.views.components

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.features.swap.viewmodels.models.SwapUiState
import uniffi.gemstone.GemSwapButtonAction

@Composable
internal fun SwapAction(
    swapState: SwapUiState,
    pay: AssetInfo?,
    onSwap: () -> Unit,
) {
    MainActionButton(
        state = swapState.buttonState,
        onClick = onSwap,
    ) {
        if (swapState.buttonState == ButtonState.Loading) {
            CircularProgressIndicator20(color = Color.White)
        } else {
            Text(
                modifier = Modifier.padding(paddingHalfSmall),
                text = when (swapState.buttonAction) {
                    GemSwapButtonAction.InsufficientBalance ->
                        stringResource(R.string.transfer_insufficient_balance, pay?.asset?.symbol ?: "")
                    is GemSwapButtonAction.UseMinimumAmount -> stringResource(R.string.swap_use_minimum_amount)
                    GemSwapButtonAction.RetryQuote,
                    GemSwapButtonAction.RetryTransfer -> stringResource(R.string.common_try_again)
                    GemSwapButtonAction.Swap -> stringResource(R.string.wallet_swap)
                },
                style = MaterialTheme.typography.bodyLarge,
            )
        }
    }
}
