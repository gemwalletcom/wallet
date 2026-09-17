package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
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
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val clipboardManager = context.clipboardManager()

    when (val row = row.uiModel(context)) {
        is GemListRowUIModel.Item -> ListItem(
            model = row.model,
            listPosition = listPosition,
            modifier = row.url?.let { url -> Modifier.clickable { uriHandler.open(context, url) } } ?: Modifier,
            accessory = row.url?.let { { DataBadgeChevron() } },
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
