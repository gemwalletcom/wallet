package com.gemwallet.android.features.swap.views.dialogs

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.swap.SwapPriceImpactUIModel
import com.wallet.core.primitives.Asset

@Composable
internal fun PriceImpactWarningDialog(
    isVisible: Boolean,
    priceImpact: SwapPriceImpactUIModel?,
    asset: Asset?,
    onDismiss: () -> Unit,
    onContinue: () -> Unit,
) {
    if (!isVisible || priceImpact == null || asset == null) {
        return
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        confirmButton = {
            Button(
                {
                    onContinue()
                    onDismiss()
                }
            ) {
                Text(stringResource(R.string.common_continue))
            }
        },
        dismissButton = {
            Button(onDismiss) {
                Text(stringResource(R.string.common_cancel))
            }
        },
        title = { Text(stringResource(R.string.swap_price_impact_warning_title)) },
        text = {
            Text(
                stringResource(
                    R.string.swap_price_impact_warning_description,
                    priceImpact.warningText,
                    asset.symbol,
                ),
            )
        },
        containerColor = MaterialTheme.colorScheme.background,
    )
}
