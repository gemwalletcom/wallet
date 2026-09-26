package com.gemwallet.android.features.support.viewmodels

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.format.gemDay
import com.gemwallet.android.ui.format.localDate
import com.wallet.core.primitives.SupportMessage
import uniffi.gemstone.GemSupportChatGroup
import uniffi.gemstone.supportChatGroups
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId

data class SupportChatDay(val id: String, val date: LocalDate, val groups: List<GemSupportChatGroup>)

fun buildSupportChatDays(messages: List<SupportMessage>, zone: ZoneId = ZoneId.systemDefault()): List<SupportChatDay> =
    LocalDate.now(zone).gemDay().boundaries().sections(messages.map { Instant.ofEpochMilli(it.createdAt).atZone(zone).toLocalDate().gemDay() }, false).map { section ->
        val date = section.day.localDate()
        SupportChatDay(id = date.toString(), date = date, groups = supportChatGroups(section.positions.map { messages[it.toInt()].toGem() }))
    }
