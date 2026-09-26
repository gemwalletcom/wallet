package com.gemwallet.android.features.perpetuals.presents.components

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
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.mainActionHeight
import uniffi.gemstone.GemPerpetualButton
import uniffi.gemstone.GemPerpetualButtonRow
import uniffi.gemstone.GemValueTone

@Composable
internal fun PerpetualModifyBottomSheet(isVisible: Boolean, title: String, buttons: List<GemPerpetualButtonRow>, onDismiss: () -> Unit, onSelect: (GemPerpetualButton) -> Unit) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        title = title,
    ) {
        buttons.forEachIndexed { index, button ->
            if (index > 0) {
                HorizontalDivider()
            }
            ModifyOption(
                label = stringResource(button.button.stringRes()),
                color = if (button.tone == GemValueTone.NEGATIVE) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurface,
                onClick = {
                    onDismiss()
                    onSelect(button.button)
                },
            )
        }
        Spacer16()
    }
}

@Composable
private fun ModifyOption(label: String, onClick: () -> Unit, color: androidx.compose.ui.graphics.Color = MaterialTheme.colorScheme.onSurface) {
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
