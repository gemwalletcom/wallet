package com.gemwallet.android.features.import_wallet.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Tab
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.alpha10
import uniffi.gemstone.GemWalletImportKind

internal fun importTypeTabIndex(kind: GemWalletImportKind, tabs: List<GemWalletImportKind>): Int {
    return tabs.indexOf(kind).takeIf { it >= 0 } ?: 0
}

@Composable
internal fun ImportKindTab(
    type: GemWalletImportKind,
    selectedType: GemWalletImportKind,
    onTypeChange: (GemWalletImportKind) -> Unit,
) {
    val isSelected = type == selectedType
    Tab(
        modifier = Modifier
            .padding(horizontal = if (isSelected) 2.dp else 0.dp, vertical = 2.dp)
            .height(32.dp)
            .clip(RoundedCornerShape(4.dp))
            .background(
                if (isSelected) {
                    MaterialTheme.colorScheme.secondary.copy(alpha = alpha10)
                } else {
                    Color.Transparent
                }
            ),
        selected = isSelected,
        onClick = { onTypeChange(type) },
        text = {
            Text(
                text = when (type) {
                    GemWalletImportKind.ADDRESS -> stringResource(id = R.string.common_address)
                    GemWalletImportKind.PHRASE -> stringResource(id = R.string.common_phrase)
                    GemWalletImportKind.PRIVATE_KEY -> stringResource(id = R.string.common_private_key)
                },
                maxLines = 1,
                color = MaterialTheme.colorScheme.onSurface,
            )
        },
        selectedContentColor = MaterialTheme.colorScheme.secondary.copy(alpha = alpha10),
        unselectedContentColor = Color.Transparent,
    )
}
