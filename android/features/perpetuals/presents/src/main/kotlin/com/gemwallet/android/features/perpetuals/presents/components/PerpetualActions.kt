package com.gemwallet.android.features.perpetuals.presents.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.buttonColor
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingDefault
import uniffi.gemstone.GemPerpetualButton
import uniffi.gemstone.GemPerpetualButtonRow
import uniffi.gemstone.GemValueTone

@Composable
internal fun PerpetualActions(buttons: List<GemPerpetualButtonRow>, onSelect: (GemPerpetualButton) -> Unit) {
    Row(
        modifier = Modifier.listItem().padding(paddingDefault),
        horizontalArrangement = Arrangement.spacedBy(paddingDefault),
    ) {
        buttons.forEach { button ->
            Button(
                onClick = { onSelect(button.button) },
                colors = ButtonDefaults.buttonColors().copy(containerColor = button.tone.buttonColor()),
                modifier = Modifier.weight(1f),
            ) {
                Text(stringResource(button.button.stringRes()))
            }
        }
    }
}

@Preview
@Composable
private fun PerpetualActionsPreview() {
    WalletTheme {
        PerpetualActions(
            buttons = listOf(
                GemPerpetualButtonRow(GemPerpetualButton.LONG, GemValueTone.POSITIVE),
                GemPerpetualButtonRow(GemPerpetualButton.SHORT, GemValueTone.NEGATIVE),
            ),
            onSelect = {},
        )
    }
}
