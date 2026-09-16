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
import java.time.Clock
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.util.Locale

data class DateSection<T>(
    val label: String,
    val items: List<T>,
)

data class DateSectionLabel(
    val label: String,
    val count: Int,
)

fun dateSectionLabels(
    timestamps: List<Long>,
    zone: ZoneId,
    locale: Locale,
    formatter: SectionDateFormatter,
): List<DateSectionLabel> {
    val labels = mutableListOf<DateSectionLabel>()
    var day: LocalDate? = null
    timestamps.forEach { timestamp ->
        val itemDay = Instant.ofEpochMilli(timestamp).atZone(zone).toLocalDate()
        if (itemDay == day) {
            labels[labels.lastIndex] = labels.last().let { it.copy(count = it.count + 1) }
        } else {
            day = itemDay
            labels.add(DateSectionLabel(label = formatter.format(itemDay, locale), count = 1))
        }
    }
    return labels
}

@Composable
fun <T> rememberDateSections(
    items: List<T>,
    createdAt: (T) -> Long,
): List<DateSection<T>> {
    val todayLabel = stringResource(R.string.date_today)
    val yesterdayLabel = stringResource(R.string.date_yesterday)
    val locale = LocalConfiguration.current.locales[0]
    val timestamps = items.map(createdAt)
    val labels = remember(timestamps, todayLabel, yesterdayLabel, locale) {
        val zone = ZoneId.systemDefault()
        dateSectionLabels(
            timestamps = timestamps,
            zone = zone,
            locale = locale,
            formatter = SectionDateFormatter(todayLabel, yesterdayLabel, Clock.system(zone)),
        )
    }
    return remember(labels, items) { dateSections(labels, items) }
}

fun <T> dateSections(labels: List<DateSectionLabel>, items: List<T>): List<DateSection<T>> {
    var start = 0
    return labels.map { label ->
        val end = minOf(start + label.count, items.size)
        DateSection(label = label.label, items = items.subList(start, end)).also { start = end }
    }
}

@OptIn(ExperimentalFoundationApi::class)
fun <T> LazyListScope.dateSectionedList(
    sections: List<DateSection<T>>,
    key: (Int, T) -> Any,
    itemContent: @Composable LazyItemScope.(ListPosition, T) -> Unit,
) {
    sections.forEach { section ->
        stickyHeader {
            SubheaderItem(
                title = section.label,
                modifier = Modifier.background(MaterialTheme.colorScheme.surface),
            )
        }
        itemsPositioned(section.items, key = key, itemContent = itemContent)
    }
}
