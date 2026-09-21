package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.theme.paddingLarge

fun <T> LazyListScope.listSections(sections: List<ListSection<T>>, key: ((T) -> Any)? = null, itemContent: @Composable LazyItemScope.(position: ListPosition, item: T) -> Unit) {
    sections.forEach { section ->
        section.title?.let { title ->
            item(key = "section:${section.id}") { SubheaderItem(title) }
        }
        itemsPositioned(section.items, key = key?.let { itemKey -> { _, item -> itemKey(item) } }) { position, item ->
            itemContent(position, item)
        }
        section.footer?.let { footer ->
            item(key = "footer:${section.id}") {
                Text(
                    modifier = Modifier.padding(horizontal = paddingLarge),
                    text = footer,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.secondary,
                )
            }
        }
    }
}

fun LazyListScope.listSections(sections: List<ListSection<ListItemModel>>) {
    listSections(sections) { position, item -> ListItem(model = item, listPosition = position) }
}
