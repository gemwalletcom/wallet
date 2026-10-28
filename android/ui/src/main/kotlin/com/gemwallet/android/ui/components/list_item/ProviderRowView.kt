package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.components.image.listItemImage
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.listItemIconSize
import uniffi.gemstone.GemProviderKind
import uniffi.gemstone.GemProviderLoadingRow
import uniffi.gemstone.GemProviderRow

@Composable
fun ProviderRowView(row: GemProviderRow, listPosition: ListPosition, onClick: (() -> Unit)? = null, isShowChevron: Boolean = false) {
    val image = row.kind.listItemImage()
    ListItem(
        modifier = onClick?.let { Modifier.clickable(onClick = it) } ?: Modifier,
        leading = {
            if (row.isSelected) {
                IconWithBadge(image = image, size = listItemIconSize, badge = { SelectionCheckmark() })
            } else {
                ListItemImageView(image, size = listItemIconSize)
            }
        },
        title = { ListItemTitleText(row.name) },
        trailing = {
            Column(horizontalAlignment = Alignment.End) {
                ListItemTitleText(row.amount.text())
                row.fiat?.let { ListItemSupportText(it.text()) }
            }
            if (isShowChevron) {
                Spacer(modifier = Modifier.width(ChevronIconDefaults.leadingSpacing))
                ChevronIcon()
            }
        },
        listPosition = listPosition,
    )
}

@Composable
fun ProviderLoadingRowView(row: GemProviderLoadingRow, listPosition: ListPosition) {
    ListItem(
        leading = { ListItemImageView(row.kind.listItemImage(), size = listItemIconSize) },
        title = { ListItemTitleText(row.name) },
        trailing = { CircularProgressIndicator20() },
        listPosition = listPosition,
    )
}

private fun GemProviderKind.listItemImage(): ListItemImage = when (this) {
    is GemProviderKind.Swap -> provider.listItemImage()
    is GemProviderKind.Fiat -> provider.toPrimitives().listItemImage()
}
