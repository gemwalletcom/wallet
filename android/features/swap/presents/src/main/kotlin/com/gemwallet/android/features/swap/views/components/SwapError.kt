package com.gemwallet.android.features.swap.views.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Warning
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.Spacer2
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.smallIconSize
import com.gemwallet.android.features.swap.viewmodels.models.SwapError
import com.gemwallet.android.features.swap.viewmodels.models.SwapUiState

@Composable
internal fun SwapError(state: SwapUiState, pay: AssetInfo?) {
    val error = state.error ?: return

    val errorText = when (error) {
        SwapError.None,
        is SwapError.InsufficientBalance -> return
        SwapError.IncorrectInput -> stringResource(
            R.string.common_required_field,
            stringResource(R.string.swap_you_pay)
        )
        SwapError.NotSupportedAsset -> stringResource(R.string.errors_swap_not_supported_asset)
        SwapError.NotSupportedChain -> stringResource(R.string.errors_swap_not_supported_chain)
        is SwapError.Unknown -> "${stringResource(R.string.errors_unknown_try_again)}: ${error.data}"
        is SwapError.InputAmountTooSmall -> "${stringResource(R.string.errors_swap_amount_too_small)} ${pay?.asset?.let { error.getFormattedValue(it) } ?: ""}"
        SwapError.NoAvailableProvider,
        SwapError.NoQuote,
        SwapError.TransactionError -> stringResource(R.string.errors_swap_no_quote_available)
    }
    Column(
        modifier = Modifier
            .defaultPadding()
            .background(
                MaterialTheme.colorScheme.errorContainer.copy(0.2f),
                shape = MaterialTheme.shapes.medium
            )
            .fillMaxWidth()
            .defaultPadding(),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                modifier = Modifier.size(smallIconSize),
                imageVector = Icons.Outlined.Warning,
                tint = MaterialTheme.colorScheme.error,
                contentDescription = null,
            )
            Spacer8()
            Text(
                text = stringResource(R.string.errors_error_occured),
                style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.W400),
            )
        }
        Spacer2()
        Text(
            text = errorText,
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}
