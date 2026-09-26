package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.listItemIconSize
import uniffi.gemstone.GemProviderKind
import uniffi.gemstone.GemProviderRow

@Composable
fun ProviderRowView(row: GemProviderRow, listPosition: ListPosition, onClick: (() -> Unit)? = null) {
    val icon = row.kind.iconModel()
    ListItem(
        modifier = onClick?.let { Modifier.clickable(onClick = it) } ?: Modifier,
        leading = {
            if (row.isSelected) {
                IconWithBadge(icon = icon, size = listItemIconSize, badge = { SelectionCheckmark() })
            } else {
                AsyncImage(model = icon, size = listItemIconSize)
            }
        },
        title = { ListItemTitleText(row.name) },
        trailing = {
            Column(horizontalAlignment = Alignment.End) {
                ListItemTitleText(row.amount.text())
                row.fiat?.let { ListItemSupportText(it.text()) }
            }
        },
        listPosition = listPosition,
    )
}

private fun GemProviderKind.iconModel(): Any = when (this) {
    is GemProviderKind.Swap -> provider.iconModel()
    is GemProviderKind.Fiat -> provider.toPrimitives().iconModel()
}
