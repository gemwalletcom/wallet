package com.gemwallet.android.features.swap.presents.components

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.WarningItem
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemSwapQuotesState
import uniffi.gemstone.GemSwapViewState

@Composable
internal fun SwapQuoteError(viewState: GemSwapViewState) {
    var isShowInfoSheet by rememberSaveable { mutableStateOf(false) }
    val error = (viewState.quotesState as? GemSwapQuotesState.Failed)?.error ?: return
    val errorText = error.text(LocalContext.current)
    val infoSheetEntity = error.info()?.infoSheet()

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
