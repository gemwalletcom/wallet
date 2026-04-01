package com.gemwallet.android.ui.components.buttons

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.space12
import com.gemwallet.android.ui.theme.toolbarButtonSize
import com.gemwallet.android.ui.theme.WalletTheme

@Composable
fun ToolbarIconButton(
    imageVector: ImageVector,
    modifier: Modifier = Modifier,
    contentDescription: String? = null,
    enabled: Boolean = true,
    tint: Color? = null,
    onClick: () -> Unit,
) {
    FilledIconButton(
        modifier = modifier.size(toolbarButtonSize),
        onClick = onClick,
        enabled = enabled,
        colors = IconButtonDefaults.filledIconButtonColors(
            containerColor = MaterialTheme.colorScheme.surfaceContainerHighest,
            contentColor = tint ?: MaterialTheme.colorScheme.onSurface,
            disabledContainerColor = MaterialTheme.colorScheme.surfaceContainerHighest.copy(alpha = 0.5f),
            disabledContentColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.38f),
        ),
    ) {
        Icon(
            modifier = Modifier.size(compactIconSize),
            imageVector = imageVector,
            contentDescription = contentDescription,
        )
    }
}

@Composable
fun ToolbarCloseButton(
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
) {
    ToolbarIconButton(
        modifier = modifier,
        imageVector = Icons.Default.Close,
        onClick = onClick,
    )
}

@Composable
fun ToolbarCheckmarkButton(
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    onClick: () -> Unit,
) {
    ToolbarIconButton(
        modifier = modifier,
        imageVector = Icons.Default.Check,
        enabled = enabled,
        onClick = onClick,
    )
}

@Preview(showBackground = true, name = "Toolbar Buttons - Light")
@Composable
private fun ToolbarButtonsLightPreview() {
    WalletTheme(darkTheme = false) {
        Row(
            modifier = Modifier.padding(paddingDefault),
            horizontalArrangement = Arrangement.spacedBy(space12),
        ) {
            ToolbarCheckmarkButton(onClick = {})
            ToolbarCloseButton(onClick = {})
        }
    }
}

@Preview(showBackground = true, backgroundColor = 0xFF1A191A, name = "Toolbar Buttons - Dark")
@Composable
private fun ToolbarButtonsDarkPreview() {
    WalletTheme(darkTheme = true) {
        Row(
            modifier = Modifier.padding(paddingDefault),
            horizontalArrangement = Arrangement.spacedBy(space12),
        ) {
            ToolbarCheckmarkButton(onClick = {})
            ToolbarCloseButton(onClick = {})
        }
    }
}
