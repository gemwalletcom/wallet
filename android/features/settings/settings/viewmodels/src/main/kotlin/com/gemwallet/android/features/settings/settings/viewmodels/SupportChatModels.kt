package com.gemwallet.android.features.settings.settings.viewmodels

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageSender
import uniffi.gemstone.parseSupportMessageDisplayContent
import uniffi.gemstone.supportChatGroups
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId

data class SupportChatDay(val id: String, val date: LocalDate, val groups: List<SupportChatGroup>)

data class SupportChatGroup(val sender: SupportMessageSender, val messages: List<SupportChatMessage>)

data class SupportChatMessage(val message: SupportMessage, val text: String, val links: List<SupportChatLink>) {
    val id: String get() = message.id
}

data class SupportChatLink(val title: String, val url: String, val subtitle: String?)

private fun SupportMessage.chatMessage(): SupportChatMessage {
    val content = parseSupportMessageDisplayContent(content)
    return SupportChatMessage(
        message = this,
        text = content.text,
        links = content.links.map { SupportChatLink(title = it.title, url = it.url, subtitle = it.subtitle) },
    )
}

fun buildSupportChatDays(messages: List<SupportMessage>): List<SupportChatDay> {
    val zone = ZoneId.systemDefault()
    return messages
        .groupBy { Instant.ofEpochMilli(it.createdAt).atZone(zone).toLocalDate() }
        .toSortedMap()
        .map { (date, dayMessages) ->
            val groups = supportChatGroups(dayMessages.map { it.toGem() }).map { group ->
                SupportChatGroup(sender = group.sender.toPrimitives(), messages = group.messages.map { it.toPrimitives().chatMessage() })
            }
            SupportChatDay(id = date.toString(), date = date, groups = groups)
        }
}
