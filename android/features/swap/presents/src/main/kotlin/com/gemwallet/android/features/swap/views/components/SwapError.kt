package com.gemwallet.android.features.swap.views.components

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.WarningItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.features.swap.viewmodels.models.SwapError
import com.gemwallet.android.features.swap.viewmodels.models.SwapUiState

@Composable
internal fun SwapError(state: SwapUiState, pay: AssetInfo?) {
    var isShowInfoSheet by remember { mutableStateOf(false) }
    val error = state.error ?: return

    val errorText = when (error) {
        SwapError.None -> return
        SwapError.NotSupportedAsset -> stringResource(R.string.errors_swap_not_supported_asset)
        is SwapError.Unknown -> "${stringResource(R.string.errors_unknown_try_again)}: ${error.data}"
        is SwapError.InputAmountTooSmall -> "${stringResource(R.string.errors_swap_amount_too_small)} ${pay?.asset?.let { error.getFormattedValue(it) } ?: ""}"
        SwapError.NoQuote -> stringResource(R.string.errors_swap_no_quote_available)
    }

    val infoSheetEntity = when (error) {
        SwapError.NoQuote -> InfoSheetEntity.NoQuoteInfo
        else -> null
    }

    WarningItem(
        title = stringResource(R.string.errors_error_occurred),
        message = errorText,
        color = MaterialTheme.colorScheme.error,
        position = ListPosition.Single,
        onClick = infoSheetEntity?.let { { isShowInfoSheet = true } },
    )

    if (isShowInfoSheet && infoSheetEntity != null) {
        InfoBottomSheet(item = infoSheetEntity) { isShowInfoSheet = false }
    }
}
