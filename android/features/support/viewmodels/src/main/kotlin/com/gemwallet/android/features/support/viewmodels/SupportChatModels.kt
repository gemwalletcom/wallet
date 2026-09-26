package com.gemwallet.android.features.support.viewmodels

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageSender
import uniffi.gemstone.GemSupportMessageOutcome
import uniffi.gemstone.GemSupportMessageRow
import uniffi.gemstone.SupportMessageLink
import uniffi.gemstone.supportChatGroups
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId

data class SupportChatDay(val id: String, val date: LocalDate, val groups: List<SupportChatGroup>)

data class SupportChatGroup(val sender: SupportMessageSender, val messages: List<SupportChatMessage>)

class SupportChatMessage(row: GemSupportMessageRow) {
    val message: SupportMessage = row.message.toPrimitives()
    val text: String = row.content.text
    val links: List<SupportMessageLink> = row.content.links
    val outcome: GemSupportMessageOutcome = row.outcome
    val id: String get() = message.id
}

fun buildSupportChatDays(messages: List<SupportMessage>): List<SupportChatDay> {
    val zone = ZoneId.systemDefault()
    return messages
        .groupBy { Instant.ofEpochMilli(it.createdAt).atZone(zone).toLocalDate() }
        .toSortedMap()
        .map { (date, dayMessages) ->
            val groups = supportChatGroups(dayMessages.map { it.toGem() }).map { group ->
                SupportChatGroup(sender = group.sender.toPrimitives(), messages = group.rows.map(::SupportChatMessage))
            }
            SupportChatDay(id = date.toString(), date = date, groups = groups)
        }
}
