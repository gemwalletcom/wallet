package com.gemwallet.android.features.settings.settings.viewmodels

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageSender
import uniffi.gemstone.supportChatGroups
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId

data class SupportChatDay(
    val id: String,
    val date: LocalDate,
    val groups: List<SupportChatGroup>,
)

data class SupportChatGroup(
    val sender: SupportMessageSender,
    val messages: List<SupportMessage>,
)

fun buildSupportChatDays(messages: List<SupportMessage>): List<SupportChatDay> {
    val zone = ZoneId.systemDefault()
    return messages
        .groupBy { Instant.ofEpochMilli(it.createdAt).atZone(zone).toLocalDate() }
        .toSortedMap()
        .map { (date, dayMessages) ->
            val groups = supportChatGroups(dayMessages.map { it.toGem() }).map { group ->
                SupportChatGroup(sender = group.sender.toPrimitives(), messages = group.messages.map { it.toPrimitives() })
            }
            SupportChatDay(id = date.toString(), date = date, groups = groups)
        }
}
