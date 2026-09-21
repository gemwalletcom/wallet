package com.gemwallet.android.ui.components.list_item

import com.gemwallet.android.ui.format.SectionDateFormatter
import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.Clock
import java.time.ZoneId
import java.time.ZonedDateTime
import java.util.Locale

class DateSectionsTest {

    private val zone = ZoneId.of("America/New_York")
    private val clock = Clock.fixed(ZonedDateTime.of(2026, 3, 8, 12, 0, 0, 0, zone).toInstant(), zone)
    private val formatter = SectionDateFormatter(todayLabel = "Today", yesterdayLabel = "Yesterday", clock = clock)

    private fun at(year: Int, month: Int, day: Int, hour: Int, minute: Int = 0): Long = ZonedDateTime.of(year, month, day, hour, minute, 0, 0, zone).toInstant().toEpochMilli()

    private fun labels(timestamps: List<Long>) = dateSectionLabels(timestamps, zone, Locale.US, formatter)

    @Test
    fun midnightStartsTheNextSectionInTheDeviceZone() {
        val sections = labels(listOf(at(2026, 3, 8, 0, 0), at(2026, 3, 7, 23, 59)))

        assertEquals(listOf("Today" to 1, "Yesterday" to 1), sections.map { it.label to it.count })
    }

    @Test
    fun aDaylightSavingSwitchKeepsOneSectionPerLocalDay() {
        val sections = labels(listOf(at(2026, 3, 8, 3, 30), at(2026, 3, 8, 1, 30), at(2026, 3, 7, 12)))

        assertEquals(listOf("Today" to 2, "Yesterday" to 1), sections.map { it.label to it.count })
    }

    @Test
    fun equalTimestampsShareOneSection() {
        val same = at(2026, 3, 6, 9)
        val sections = labels(listOf(same, same, same))

        assertEquals(listOf("March 6, 2026" to 3), sections.map { it.label to it.count })
    }

    @Test
    fun sectionsSliceTheCurrentRowsInOrder() {
        val rows = listOf("today", "yesterday-a", "yesterday-b")
        val sections = dateSections(
            labels = listOf(DateSectionLabel("Today", 1), DateSectionLabel("Yesterday", 2)),
            items = rows,
        )

        assertEquals(listOf("Today", "Yesterday"), sections.map { it.label })
        assertEquals(listOf(listOf("today"), listOf("yesterday-a", "yesterday-b")), sections.map { it.items })
    }

    @Test
    fun reusedLabelsBindTheLatestRowValues() {
        val labels = labels(listOf(at(2026, 3, 8, 10), at(2026, 3, 7, 10)))

        val refreshed = dateSections(labels, listOf("updated price", "yesterday"))

        assertEquals(listOf(listOf("updated price"), listOf("yesterday")), refreshed.map { it.items })
    }

    @Test
    fun anEmptyListHasNoSections() {
        assertEquals(emptyList<DateSectionLabel>(), labels(emptyList()))
        assertEquals(emptyList<DateSection<String>>(), dateSections(emptyList(), emptyList<String>()))
    }
}
