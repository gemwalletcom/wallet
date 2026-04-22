package com.gemwallet.android.ui.components.list_item

import androidx.annotation.StringRes
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.compactIconSize

@Composable
fun SubheaderItem(@StringRes title: Int, vararg formatArgs: Any) {
    SubheaderItem(stringResource(title, formatArgs))
}

@Composable
fun SubheaderItem(title: String, modifier: Modifier = Modifier) {
    Text(
        modifier = modifier
            .sectionHeaderItem(),
        text = title,
        style = MaterialTheme.typography.labelLarge,
        color = MaterialTheme.colorScheme.secondary,
    )
}

@Composable
fun SubheaderItem(@StringRes title: Int, onClick: () -> Unit) {
    SubheaderItem(stringResource(title), onClick)
}

@Composable
fun SubheaderItem(title: String, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .sectionHeaderItem()
            .clickable(onClick = onClick),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.secondary,
        )
        Spacer4()
        ChevronIcon()
    }
}

object ChevronIconDefaults {
    val trailingEdgeNudge: Dp = 6.dp
    val leadingSpacing: Dp = 0.dp
    val size: Dp = compactIconSize + 4.dp
}

@Composable
fun ChevronIcon(
    modifier: Modifier = Modifier,
    size: Dp = ChevronIconDefaults.size,
    tint: Color = MaterialTheme.colorScheme.secondary,
) {
    Icon(
        imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
        contentDescription = null,
        modifier = modifier
            .offset(x = ChevronIconDefaults.trailingEdgeNudge)
            .size(size),
        tint = tint,
    )
}
