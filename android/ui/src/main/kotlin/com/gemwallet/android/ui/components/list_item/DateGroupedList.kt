package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.format.SectionDateFormatter
import com.gemwallet.android.ui.models.ListPosition
import java.time.Instant
import java.time.ZoneId

@OptIn(ExperimentalFoundationApi::class)
fun <T> LazyListScope.dateGroupedList(
    items: List<T>,
    createdAt: (T) -> Long,
    key: (Int, T) -> Any,
    itemContent: @Composable LazyItemScope.(ListPosition, T) -> Unit,
) {
    val zone = ZoneId.systemDefault()
    items.groupBy { Instant.ofEpochMilli(createdAt(it)).atZone(zone).toLocalDate() }
        .forEach { (date, entries) ->
            stickyHeader {
                val todayLabel = stringResource(R.string.date_today)
                val yesterdayLabel = stringResource(R.string.date_yesterday)
                val formatter = remember(todayLabel, yesterdayLabel) {
                    SectionDateFormatter(todayLabel, yesterdayLabel)
                }
                SubheaderItem(
                    title = formatter.format(date, LocalConfiguration.current.locales[0]),
                    modifier = Modifier.background(MaterialTheme.colorScheme.surface),
                )
            }
            itemsPositioned(entries, key = key, itemContent = itemContent)
        }
}
