package com.gemwallet.android.features.perpetual.views.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.features.perpetual.localization.stringRes
import uniffi.gemstone.GemPerpetualButton

@Composable
internal fun PerpetualActions(
    buttons: List<GemPerpetualButton>,
    onSelect: (GemPerpetualButton) -> Unit,
) {
    Row(
        modifier = Modifier.listItem().padding(paddingDefault),
        horizontalArrangement = Arrangement.spacedBy(paddingDefault),
    ) {
        buttons.forEach { button ->
            Button(
                onClick = { onSelect(button) },
                colors = ButtonDefaults.buttonColors().copy(containerColor = button.containerColor()),
                modifier = Modifier.weight(1f),
            ) {
                Text(stringResource(button.stringRes()))
            }
        }
    }
}

@Composable
private fun GemPerpetualButton.containerColor() = when (this) {
    GemPerpetualButton.LONG -> MaterialTheme.colorScheme.tertiary
    GemPerpetualButton.SHORT, GemPerpetualButton.CLOSE, GemPerpetualButton.REDUCE -> MaterialTheme.colorScheme.error
    GemPerpetualButton.MODIFY, GemPerpetualButton.INCREASE -> MaterialTheme.colorScheme.primary
}

@Preview
@Composable
private fun PerpetualActionsPreview() {
    WalletTheme {
        PerpetualActions(
            buttons = listOf(GemPerpetualButton.LONG, GemPerpetualButton.SHORT),
            onSelect = {},
        )
    }
}
