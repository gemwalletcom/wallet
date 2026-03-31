package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.LocalIndication
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.theme.paddingDefault

@Composable
fun DropDownContextItem(
    modifier: Modifier = Modifier,
    isExpanded: Boolean,
    onDismiss: () -> Unit,
    content: @Composable (modifier: Modifier) -> Unit,
    menuItems: @Composable ColumnScope.() -> Unit,
    onLongClick: () -> Unit,
    onClick: () -> Unit,
) {
    val interactionSource = remember { MutableInteractionSource() }

    Box {
        content(
            modifier
                .combinedClickable(
                    interactionSource = interactionSource,
                    indication = LocalIndication.current,
                    onClick = onClick,
                    onLongClick = onLongClick,
                )
        )
        DropdownMenu(
            modifier = Modifier.align(Alignment.TopEnd),
            expanded = isExpanded,
            offset = DpOffset(x = -paddingDefault, y = 0.dp),
            containerColor = MaterialTheme.colorScheme.background,
            onDismissRequest = onDismiss,
            content = menuItems,
        )
    }
}
