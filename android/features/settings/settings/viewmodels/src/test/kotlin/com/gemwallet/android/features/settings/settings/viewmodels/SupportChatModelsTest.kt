package com.gemwallet.android.features.settings.settings.viewmodels

import com.wallet.core.primitives.SupportAgent
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageSender
import com.wallet.core.primitives.SupportMessageStatus
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class SupportChatModelsTest {

    private val day1 = 1_749_643_200_000L
    private val day2 = day1 + 2 * 86_400_000L

    private val user = SupportMessageSender.User

    private fun agent(name: String) = SupportMessageSender.Agent(SupportAgent(name))

    private fun message(id: String, sender: SupportMessageSender, createdAt: Long) = SupportMessage(
        id = id,
        content = id,
        sender = sender,
        status = SupportMessageStatus.Sent,
        createdAt = createdAt,
        images = emptyList(),
    )

    @Test
    fun emptyInputReturnsNoDays() {
        assertEquals(emptyList<SupportChatDay>(), buildSupportChatDays(emptyList()))
    }

    @Test
    fun groupsByDaySortedAscending() {
        val days = buildSupportChatDays(listOf(message("b", user, day2), message("a", user, day1)))

        assertEquals(2, days.size)
        assertEquals(listOf("a"), days[0].groups.flatMap { it.messages }.map { it.id })
        assertEquals(listOf("b"), days[1].groups.flatMap { it.messages }.map { it.id })
    }

    @Test
    fun consecutiveSameSenderMergesIntoOneGroup() {
        val days = buildSupportChatDays(
            listOf(message("a", user, day1), message("b", user, day1 + 3_600_000L)),
        )

        assertEquals(1, days[0].groups.size)
        assertEquals(listOf("a", "b"), days[0].groups[0].messages.map { it.id })
    }

    @Test
    fun senderChangeAndAgentNameStartNewGroups() {
        val days = buildSupportChatDays(
            listOf(
                message("a", user, day1),
                message("b", agent("Ann"), day1 + 1_000L),
                message("c", agent("Bob"), day1 + 2_000L),
            ),
        )

        assertEquals(3, days[0].groups.size)
        assertTrue(days[0].groups[0] is SupportChatGroup.User)
        assertEquals("Ann", (days[0].groups[1] as SupportChatGroup.Agent).name)
        assertEquals("Bob", (days[0].groups[2] as SupportChatGroup.Agent).name)
    }

    @Test
    fun nonConsecutiveSameAgentProducesSeparateGroups() {
        val days = buildSupportChatDays(
            listOf(
                message("a", agent("Ann"), day1),
                message("b", user, day1 + 1_000L),
                message("c", agent("Ann"), day1 + 2_000L),
            ),
        )

        assertEquals(3, days[0].groups.size)
        assertEquals(listOf("a"), days[0].groups[0].messages.map { it.id })
        assertEquals(listOf("b"), days[0].groups[1].messages.map { it.id })
        assertEquals(listOf("c"), days[0].groups[2].messages.map { it.id })
    }
}
