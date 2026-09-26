package com.gemwallet.android.features.swap.presents.components

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.swap.viewmodels.models.SwapUIState
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.theme.paddingHalfSmall

@Composable
internal fun SwapButton(swapState: SwapUIState, pay: AssetInfo?, onSwap: () -> Unit) {
    MainActionButton(
        state = swapState.buttonState,
        onClick = onSwap,
    ) {
        if (swapState.buttonState == ButtonState.Loading) {
            CircularProgressIndicator20(color = Color.White)
        } else {
            Text(
                modifier = Modifier.padding(paddingHalfSmall),
                text = stringResource(swapState.actionTitle, pay?.asset?.symbol ?: ""),
                style = MaterialTheme.typography.bodyLarge,
            )
        }
    }
}
