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
    private val formatter = SectionDateFormatter(todayLabel = "Today", yesterdayLabel = "Yesterday", dayTimeTemplate = "%1\$s, %2\$s", clock = clock)

    private fun at(year: Int, month: Int, day: Int, hour: Int, minute: Int = 0): Long = ZonedDateTime.of(year, month, day, hour, minute, 0, 0, zone).toInstant().toEpochMilli()

    private fun sections(timestamps: List<Long>) = dateSections(timestamps, { it }, zone, Locale.US, formatter)

    @Test
    fun midnightStartsTheNextSectionInTheDeviceZone() {
        val sections = sections(listOf(at(2026, 3, 8, 0, 0), at(2026, 3, 7, 23, 59)))

        assertEquals(listOf("Today" to 1, "Yesterday" to 1), sections.map { it.label to it.items.size })
    }

    @Test
    fun aDaylightSavingSwitchKeepsOneSectionPerLocalDay() {
        val sections = sections(listOf(at(2026, 3, 8, 3, 30), at(2026, 3, 8, 1, 30), at(2026, 3, 7, 12)))

        assertEquals(listOf("Today" to 2, "Yesterday" to 1), sections.map { it.label to it.items.size })
    }

    @Test
    fun equalTimestampsShareOneSection() {
        val same = at(2026, 3, 6, 9)
        val sections = sections(listOf(same, same, same))

        assertEquals(listOf("March 6, 2026" to 3), sections.map { it.label to it.items.size })
    }

    @Test
    fun sectionsKeepTheRowsInOrderAndNewestDayFirst() {
        val rows = listOf("yesterday-a" to at(2026, 3, 7, 10), "today" to at(2026, 3, 8, 10), "yesterday-b" to at(2026, 3, 7, 9))
        val sections = dateSections(rows, { it.second }, zone, Locale.US, formatter)

        assertEquals(listOf("Today", "Yesterday"), sections.map { it.label })
        assertEquals(listOf(listOf("today"), listOf("yesterday-a", "yesterday-b")), sections.map { section -> section.items.map { it.first } })
    }

    @Test
    fun anEmptyListHasNoSections() {
        assertEquals(emptyList<DateSection<Long>>(), sections(emptyList()))
    }
}
