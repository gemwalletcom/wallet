package com.gemwallet.android.features.settings.settings.viewmodels

import com.gemwallet.android.testkit.mockSupportMessage
import com.wallet.core.primitives.SupportAgent
import com.wallet.core.primitives.SupportMessageSender
import org.junit.Assert.assertEquals
import org.junit.Test

class SupportChatModelsTest {

    private val day1 = 1_749_643_200_000L
    private val day2 = day1 + 2 * 86_400_000L

    private val user = SupportMessageSender.User

    private val ann = SupportMessageSender.Agent(SupportAgent("Ann"))

    @Test
    fun emptyInputReturnsNoDays() {
        assertEquals(emptyList<SupportChatDay>(), buildSupportChatDays(emptyList()))
    }

    @Test
    fun groupsByDaySortedAscending() {
        val days = buildSupportChatDays(listOf(mockSupportMessage("b", user, day2), mockSupportMessage("a", user, day1)))

        assertEquals(2, days.size)
        assertEquals(listOf("a"), days[0].groups.flatMap { it.messages }.map { it.id })
        assertEquals(listOf("b"), days[1].groups.flatMap { it.messages }.map { it.id })
    }

    @Test
    fun groupsCarryTheSenderAndTheMessages() {
        val days = buildSupportChatDays(
            listOf(
                mockSupportMessage("a", user, day1),
                mockSupportMessage("b", ann, day1 + 1_000L),
                mockSupportMessage("c", ann, day1 + 2_000L),
            ),
        )

        assertEquals(listOf(user, ann), days[0].groups.map { it.sender })
        assertEquals(listOf(listOf("a"), listOf("b", "c")), days[0].groups.map { group -> group.messages.map { it.id } })
    }
}
