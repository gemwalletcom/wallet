package com.gemwallet.android.features.swap.views.components

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.swap.viewmodels.models.SwapUiState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.WarningItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.features.swap.localization.text
import uniffi.gemstone.GemSwapErrorDisplay

@Composable
internal fun SwapError(state: SwapUiState) {
    var isShowInfoSheet by remember { mutableStateOf(false) }
    val error = state.error ?: return

    val infoSheetEntity = when (error) {
        is GemSwapErrorDisplay.NoQuote -> InfoSheetEntity.NoQuoteInfo
        is GemSwapErrorDisplay.NotSupportedAsset,
        is GemSwapErrorDisplay.MinimumAmount,
        is GemSwapErrorDisplay.AmountTooSmall -> null
    }

    WarningItem(
        title = stringResource(R.string.errors_error_occurred),
        message = error.text(),
        color = MaterialTheme.colorScheme.error,
        position = ListPosition.Single,
        onClick = infoSheetEntity?.let { { isShowInfoSheet = true } },
    )

    if (isShowInfoSheet && infoSheetEntity != null) {
        InfoBottomSheet(item = infoSheetEntity) { isShowInfoSheet = false }
    }
}
