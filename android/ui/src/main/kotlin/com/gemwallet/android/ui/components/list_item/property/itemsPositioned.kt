package com.gemwallet.android.ui.components.list_item.property

import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.models.ListPosition

inline fun <T> LazyListScope.itemsPositioned(
    items: List<T>,
    indexOffset: Int = 0,
    totalCount: Int = items.size,
    noinline key: ((index: Int, item: T) -> Any)? = null,
    crossinline contentType: (index: Int, item: T) -> Any? = { _, _ -> null },
    crossinline itemContent: @Composable LazyItemScope.(position: ListPosition, item: T) -> Unit,
) {
    itemsIndexed(items, key, contentType) { index, item ->
        val position = ListPosition.getPosition(indexOffset + index, totalCount)
        itemContent(position, item)
    }
}