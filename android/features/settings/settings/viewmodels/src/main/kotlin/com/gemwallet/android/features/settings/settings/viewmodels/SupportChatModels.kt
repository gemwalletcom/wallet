package com.gemwallet.android.features.settings.settings.viewmodels

import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageSender
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId

data class SupportChatDay(
    val id: String,
    val date: LocalDate,
    val groups: List<SupportChatGroup>,
)

sealed interface SupportChatGroup {
    val messages: List<SupportMessage>

    data class User(override val messages: List<SupportMessage>) : SupportChatGroup
    data class Agent(val name: String, override val messages: List<SupportMessage>) : SupportChatGroup
}

fun buildSupportChatDays(messages: List<SupportMessage>): List<SupportChatDay> {
    val zone = ZoneId.systemDefault()
    return messages
        .groupBy { Instant.ofEpochMilli(it.createdAt).atZone(zone).toLocalDate() }
        .toSortedMap()
        .map { (date, dayMessages) ->
            SupportChatDay(id = date.toString(), date = date, groups = groupBySender(dayMessages))
        }
}

private fun groupBySender(messages: List<SupportMessage>): List<SupportChatGroup> {
    val groups = mutableListOf<SupportChatGroup>()
    var current = mutableListOf<SupportMessage>()
    for (message in messages) {
        if (current.isNotEmpty() && current.last().senderKey() != message.senderKey()) {
            groups.add(current.toGroup())
            current = mutableListOf()
        }
        current.add(message)
    }
    if (current.isNotEmpty()) {
        groups.add(current.toGroup())
    }
    return groups
}

private fun SupportMessage.senderKey(): String = when (val sender = sender) {
    is SupportMessageSender.Agent -> "agent-${sender.data.name}"
    SupportMessageSender.User -> "user"
}

private fun List<SupportMessage>.toGroup(): SupportChatGroup =
    when (val sender = first().sender) {
        is SupportMessageSender.Agent -> SupportChatGroup.Agent(name = sender.data.name, messages = this)
        SupportMessageSender.User -> SupportChatGroup.User(messages = this)
    }
