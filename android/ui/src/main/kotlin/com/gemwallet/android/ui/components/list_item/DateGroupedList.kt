package com.gemwallet.android.ui.components.list_item

import android.icu.util.Calendar
import android.text.format.DateUtils
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.models.ListPosition
import java.text.DateFormat
import java.util.Date

@OptIn(ExperimentalFoundationApi::class)
fun <T> LazyListScope.dateGroupedList(
    items: List<T>,
    createdAt: (T) -> Long,
    key: (Int, T) -> Any,
    itemContent: @Composable LazyItemScope.(ListPosition, T) -> Unit,
) {
    val dateFormat = DateFormat.getDateInstance(DateFormat.MEDIUM)
    val calendar = Calendar.getInstance()

    items.groupBy { item ->
        calendar.timeInMillis = createdAt(item)
        calendar[Calendar.MILLISECOND] = 999
        calendar[Calendar.SECOND] = 59
        calendar[Calendar.MINUTE] = 59
        calendar[Calendar.HOUR_OF_DAY] = 23
        calendar.time.time
    }.forEach { (timestamp, entries) ->
        stickyHeader {
            val title = if (DateUtils.isToday(timestamp) || DateUtils.isToday(timestamp + DateUtils.DAY_IN_MILLIS)) {
                DateUtils.getRelativeTimeSpanString(timestamp, System.currentTimeMillis(), DateUtils.DAY_IN_MILLIS).toString()
            } else {
                dateFormat.format(Date(timestamp))
            }
            SubheaderItem(
                title = title,
                modifier = Modifier.background(MaterialTheme.colorScheme.surface),
            )
        }
        itemsPositioned(entries, key = key, itemContent = itemContent)
    }
}
