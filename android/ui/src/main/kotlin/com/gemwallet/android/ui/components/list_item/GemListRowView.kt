package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.setCopy
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.localization.titleRes
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemListSection

fun LazyListScope.gemListSections(sections: List<GemListSection>) {
    sections.forEach { section ->
        section.title.titleRes()?.let { title ->
            item(key = "section:$title") { SubheaderItem(title) }
        }
        itemsPositioned(section.rows) { position, row -> GemListRowView(row = row, listPosition = position) }
    }
}

@Composable
fun GemListRowView(
    row: GemListRow,
    listPosition: ListPosition,
    modifier: Modifier = Modifier,
    onToggle: ((GemListRowTitle, Boolean) -> Unit)? = null,
    onSelect: ((GemListRowTitle) -> Unit)? = null,
    accessory: (@Composable () -> Unit)? = null,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val clipboardManager = context.clipboardManager()

    when (val row = row.uiModel(context)) {
        is GemListRowUIModel.Item -> ListItem(
            model = row.model,
            listPosition = listPosition,
            modifier = modifier.then(row.url?.let { url -> Modifier.clickable { uriHandler.open(context, url) } } ?: Modifier),
            minHeight = ListItemDefaults.plainMinHeight,
            accessory = if (row.url != null || row.opensAnotherScreen) {
                {
                    DataBadgeChevron()
                    accessory?.invoke()
                }
            } else {
                null
            },
        )
        is GemListRowUIModel.Icon -> Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = paddingDefault, bottom = paddingSmall),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            HeaderIcon(row.asset)
        }
        is GemListRowUIModel.Address -> AddressCard(row = row) { clipboardManager.setCopy(context, row.copy) }
        is GemListRowUIModel.Toggle -> ListItem(
            model = row.model,
            listPosition = listPosition,
            modifier = modifier,
            minHeight = ListItemDefaults.plainMinHeight,
            accessory = { Switch(checked = row.isOn, onCheckedChange = { onToggle?.invoke(row.title, it) }) },
        )
        is GemListRowUIModel.Picker -> ListItem(
            model = row.model,
            listPosition = listPosition,
            modifier = modifier.clickable { onSelect?.invoke(row.title) },
            minHeight = ListItemDefaults.plainMinHeight,
            accessory = {
                DataBadgeChevron()
                accessory?.invoke()
            },
        )
        is GemListRowUIModel.Social -> Column {
            row.links.forEachIndexed { index, link ->
                ListItem(
                    model = link.uiModel(context).model,
                    listPosition = ListPosition.getPosition(index, row.links.size),
                    modifier = Modifier.clickable { uriHandler.open(context, link.url) },
                    minHeight = ListItemDefaults.plainMinHeight,
                    accessory = { DataBadgeChevron() },
                )
            }
        }
        GemListRowUIModel.Loading -> Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(paddingDefault),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            CircularProgressIndicator16()
        }
    }
}
