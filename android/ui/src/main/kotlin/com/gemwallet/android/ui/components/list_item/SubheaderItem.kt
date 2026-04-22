package com.gemwallet.android.ui.components.list_item

import androidx.annotation.StringRes
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.tinyIconSize

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
        ChevronIcon(modifier = Modifier.size(tinyIconSize))
    }
}
