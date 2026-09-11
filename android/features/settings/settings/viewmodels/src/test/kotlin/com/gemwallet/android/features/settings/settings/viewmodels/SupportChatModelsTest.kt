package com.gemwallet.android.features.settings.settings.viewmodels

import com.wallet.core.primitives.SupportAgent
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageSender
import com.wallet.core.primitives.SupportMessageStatus
import org.junit.Assert.assertEquals
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
    fun groupsCarryTheSenderAndTheMessages() {
        val days = buildSupportChatDays(
            listOf(
                message("a", user, day1),
                message("b", agent("Ann"), day1 + 1_000L),
                message("c", agent("Ann"), day1 + 2_000L),
            ),
        )

        assertEquals(listOf(user, agent("Ann")), days[0].groups.map { it.sender })
        assertEquals(listOf(listOf("a"), listOf("b", "c")), days[0].groups.map { group -> group.messages.map { it.id } })
    }
}
