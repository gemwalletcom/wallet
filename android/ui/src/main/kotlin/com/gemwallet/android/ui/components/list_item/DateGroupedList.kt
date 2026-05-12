package com.gemwallet.android.ui.components.list_item

import android.icu.util.Calendar
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.format.SectionDateFormatter
import com.gemwallet.android.ui.models.ListPosition

@OptIn(ExperimentalFoundationApi::class)
fun <T> LazyListScope.dateGroupedList(
    items: List<T>,
    createdAt: (T) -> Long,
    key: (Int, T) -> Any,
    itemContent: @Composable LazyItemScope.(ListPosition, T) -> Unit,
) {
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
            val title = SectionDateFormatter.format(
                timestamp = timestamp,
                todayLabel = stringResource(R.string.date_today),
                yesterdayLabel = stringResource(R.string.date_yesterday),
                locale = LocalConfiguration.current.locales[0],
            )
            SubheaderItem(
                title = title,
                modifier = Modifier.background(MaterialTheme.colorScheme.surface),
            )
        }
        itemsPositioned(entries, key = key, itemContent = itemContent)
    }
}
