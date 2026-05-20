package com.gemwallet.android.features.perpetual.views.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.mainActionHeight

@Composable
internal fun PerpetualModifyBottomSheet(
    isVisible: Boolean,
    onDismiss: () -> Unit,
    onIncreasePosition: () -> Unit,
    onReducePosition: () -> Unit,
) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        title = stringResource(R.string.perpetual_modify),
    ) {
        ModifyOption(
            label = stringResource(R.string.perpetual_increase_position),
            onClick = {
                onDismiss()
                onIncreasePosition()
            },
        )
        HorizontalDivider()
        ModifyOption(
            label = stringResource(R.string.perpetual_reduce_position),
            color = MaterialTheme.colorScheme.error,
            onClick = {
                onDismiss()
                onReducePosition()
            },
        )
        Spacer16()
    }
}

@Composable
private fun ModifyOption(
    label: String,
    onClick: () -> Unit,
    color: androidx.compose.ui.graphics.Color = MaterialTheme.colorScheme.onSurface,
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(mainActionHeight)
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.bodyLarge,
            color = color,
        )
    }
}
