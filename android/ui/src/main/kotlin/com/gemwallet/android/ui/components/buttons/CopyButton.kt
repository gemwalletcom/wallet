package com.gemwallet.android.ui.components.buttons

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.R

private val iconSize = 18.dp
private val iconTextSpacing = 6.dp

@Composable
fun CopyButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    TextButton(onClick = onClick, modifier = modifier) {
        Icon(
            imageVector = Icons.Outlined.ContentCopy,
            contentDescription = null,
            modifier = Modifier.size(iconSize),
            tint = MaterialTheme.colorScheme.onSurface,
        )
        Spacer(modifier = Modifier.size(iconTextSpacing))
        Text(
            text = stringResource(id = R.string.common_copy),
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}
