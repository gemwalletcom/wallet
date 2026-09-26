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
import com.gemwallet.android.ui.format.gemDay
import com.gemwallet.android.ui.format.localDate
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemDay
import java.time.Clock
import java.time.Instant
import java.time.ZoneId
import java.util.Locale

data class DateSection<T>(val label: String, val items: List<T>)

@Composable
fun <T> rememberDateSections(items: List<T>, createdAt: (T) -> Long): List<DateSection<T>> {
    val todayLabel = stringResource(R.string.date_today)
    val yesterdayLabel = stringResource(R.string.date_yesterday)
    val locale = LocalConfiguration.current.locales[0]
    return remember(items, todayLabel, yesterdayLabel, locale) {
        val zone = ZoneId.systemDefault()
        dateSections(items, createdAt, zone, locale, SectionDateFormatter(todayLabel, yesterdayLabel, Clock.system(zone)))
    }
}

@Composable
fun <D, T> rememberDaySections(days: List<D>, day: (D) -> GemDay, items: (D) -> List<T>): List<DateSection<T>> {
    val todayLabel = stringResource(R.string.date_today)
    val yesterdayLabel = stringResource(R.string.date_yesterday)
    val locale = LocalConfiguration.current.locales[0]
    return remember(days, todayLabel, yesterdayLabel, locale) {
        val formatter = SectionDateFormatter(todayLabel, yesterdayLabel, Clock.system(ZoneId.systemDefault()))
        days.map { DateSection(label = formatter.format(day(it).localDate(), locale), items = items(it)) }
    }
}

fun <T> dateSections(items: List<T>, createdAt: (T) -> Long, zone: ZoneId, locale: Locale, formatter: SectionDateFormatter): List<DateSection<T>> =
    formatter.boundaries.sections(items.map { Instant.ofEpochMilli(createdAt(it)).atZone(zone).toLocalDate().gemDay() }, true).map { section ->
        DateSection(label = formatter.format(section.day.localDate(), locale), items = section.positions.map { items[it.toInt()] })
    }

@OptIn(ExperimentalFoundationApi::class)
fun <T> LazyListScope.dateSectionedList(sections: List<DateSection<T>>, key: (Int, T) -> Any, itemContent: @Composable LazyItemScope.(ListPosition, T) -> Unit) {
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
