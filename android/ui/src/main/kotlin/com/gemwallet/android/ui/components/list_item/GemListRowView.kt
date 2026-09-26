package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.setCopy
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_item.property.AssetRatePropertyItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.style.color
import com.gemwallet.android.ui.style.icon
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.headerIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.smallIconSize
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemListSection
import uniffi.gemstone.GemRowAction

fun LazyListScope.gemListSections(sections: List<GemListSection>, onSelectAddress: ((String) -> Unit)? = null, onSelect: ((GemRowAction) -> Unit)? = null) {
    sections.forEachIndexed { index, section ->
        val title = section.title.titleRes()
        if (title != null) {
            item(key = "section:$index") { SubheaderItem(title) }
        } else if (index > 0) {
            item(key = "section:$index") { Spacer16() }
        }
        itemsPositioned(section.rows) { position, row -> GemListRowView(row = row, listPosition = position, onSelect = onSelect, onSelectAddress = onSelectAddress) }
    }
}

@Composable
fun GemListRowView(
    row: GemListRow,
    listPosition: ListPosition,
    modifier: Modifier = Modifier,
    action: GemRowAction? = row.action(),
    onToggle: ((GemRowAction, Boolean) -> Unit)? = null,
    onSelect: ((GemRowAction) -> Unit)? = null,
    onSelectAddress: ((String) -> Unit)? = null,
    accessory: (@Composable () -> Unit)? = null,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val clipboardManager = context.clipboardManager()

    when (val row = row.uiModel(context)) {
        is GemListRowUIModel.Notice -> WarningItem(
            title = row.title,
            message = row.message,
            color = row.kind.color(),
            position = listPosition,
            icon = row.kind.icon(),
        )

        is GemListRowUIModel.Provider -> {
            val openContract = row.contract?.let { contract -> onSelectAddress?.let { select -> { select(contract) } } }
            ListItem(
                model = row.model,
                listPosition = listPosition,
                modifier = modifier.then(openContract?.let { Modifier.clickable(onClick = it) } ?: Modifier),
                minHeight = ListItemDefaults.plainMinHeight,
                accessory = if (openContract == null) accessory else ({ DataBadgeChevron() }),
            )
        }

        is GemListRowUIModel.Item -> GemListRowMenu(items = row.menu) { menuModifier ->
            val selects = onSelect != null && action != null && row.url == null
            val openAddress = row.address?.let { address -> onSelectAddress?.let { select -> { select(address) } } }
            ListItem(
                model = row.model,
                listPosition = listPosition,
                modifier = modifier.then(menuModifier).then(
                    when {
                        openAddress != null -> Modifier.clickable(onClick = openAddress)
                        row.url != null -> Modifier.clickable { uriHandler.open(context, row.url) }
                        selects -> Modifier.clickable { action?.let(onSelect) }
                        else -> Modifier
                    },
                ),
                minHeight = ListItemDefaults.plainMinHeight,
                accessory = when {
                    row.trailingImage != null -> {
                        { DataBadgeChevron(isShowChevron = false) { ListItemImageView(image = row.trailingImage, size = smallIconSize) } }
                    }

                    openAddress != null || row.url != null || row.opensAnotherScreen || selects -> {
                        {
                            DataBadgeChevron()
                            if (accessory != null) accessory()
                        }
                    }

                    else -> accessory
                },
            )
        }

        is GemListRowUIModel.Rate -> AssetRatePropertyItem(title = row.title, rate = row.rate, listPosition = listPosition)

        is GemListRowUIModel.Icon -> Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = paddingDefault, bottom = paddingSmall),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            row.imageUrl?.let { url -> IconWithBadge(icon = url, placeholder = row.icon.placeholder, size = headerIconSize) } ?: HeaderIcon(row.icon)
        }

        is GemListRowUIModel.Avatar -> Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = paddingDefault, bottom = paddingSmall),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            ListItemImageView(image = row.image, size = headerIconSize)
        }

        is GemListRowUIModel.Address -> GemListRowMenu(items = row.menu, onClick = { clipboardManager.setCopy(context, row.copy) }) { menuModifier ->
            AddressRow(address = row.address, listPosition = listPosition, modifier = menuModifier)
        }

        is GemListRowUIModel.Network -> PropertyNetworkItem(
            chain = row.chain,
            value = row.name,
            listPosition = listPosition,
            onOpenNetwork = onSelect?.let { select -> { select(GemRowAction.Network) } },
        )

        is GemListRowUIModel.Toggle -> ListItem(
            model = row.model,
            listPosition = listPosition,
            modifier = modifier,
            minHeight = ListItemDefaults.plainMinHeight,
            accessory = { Switch(checked = row.isOn, onCheckedChange = { onToggle?.invoke(row.action, it) }) },
        )

        is GemListRowUIModel.Picker -> ListItem(
            model = row.model,
            listPosition = listPosition,
            modifier = modifier.clickable { onSelect?.invoke(row.action) },
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

@Composable
private fun GemListRowMenu(items: List<GemListRowMenuItem>, onClick: () -> Unit = {}, content: @Composable (Modifier) -> Unit) {
    if (items.isEmpty()) {
        content(Modifier)
        return
    }
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val clipboardManager = context.clipboardManager()
    var isExpanded by remember { mutableStateOf(false) }

    DropDownContextItem(
        isExpanded = isExpanded,
        onDismiss = { isExpanded = false },
        onLongClick = { isExpanded = true },
        onClick = onClick,
        content = content,
        menuItems = {
            items.forEach { item ->
                DropdownMenuItem(
                    text = { Text(text = item.title) },
                    trailingIcon = {
                        when (item) {
                            is GemListRowMenuItem.Copy -> Icon(AppIcons.ContentCopy, contentDescription = null)
                            is GemListRowMenuItem.Open -> DataBadgeChevron()
                        }
                    },
                    onClick = {
                        isExpanded = false
                        when (item) {
                            is GemListRowMenuItem.Copy -> clipboardManager.setPlainText(context, item.value)
                            is GemListRowMenuItem.Open -> uriHandler.open(context, item.url)
                        }
                    },
                )
            }
        },
    )
}
