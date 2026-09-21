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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualButtonAction
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualButtonTone
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualButtonUIModel
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingDefault

@Composable
internal fun PerpetualActions(buttons: List<PerpetualButtonUIModel>, onSelect: (PerpetualButtonAction) -> Unit) {
    Row(
        modifier = Modifier.listItem().padding(paddingDefault),
        horizontalArrangement = Arrangement.spacedBy(paddingDefault),
    ) {
        buttons.forEach { button ->
            Button(
                onClick = { onSelect(button.action) },
                colors = ButtonDefaults.buttonColors().copy(containerColor = button.tone.color()),
                modifier = Modifier.weight(1f),
            ) {
                Text(button.title)
            }
        }
    }
}

@Composable
internal fun PerpetualButtonTone.color(): Color = when (this) {
    PerpetualButtonTone.Positive -> MaterialTheme.colorScheme.tertiary
    PerpetualButtonTone.Negative -> MaterialTheme.colorScheme.error
    PerpetualButtonTone.Primary -> MaterialTheme.colorScheme.primary
}

@Preview
@Composable
private fun PerpetualActionsPreview() {
    WalletTheme {
        PerpetualActions(
            buttons = listOf(
                PerpetualButtonUIModel("Long", PerpetualButtonAction.OpenLong, PerpetualButtonTone.Positive),
                PerpetualButtonUIModel("Short", PerpetualButtonAction.OpenShort, PerpetualButtonTone.Negative),
            ),
            onSelect = {},
        )
    }
}
