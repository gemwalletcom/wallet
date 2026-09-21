package com.gemwallet.android.testkit

import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageSender
import com.wallet.core.primitives.SupportMessageStatus

fun mockSupportMessage(id: String = "message-1", sender: SupportMessageSender = SupportMessageSender.User, createdAt: Long = 0L) = SupportMessage(
    id = id,
    content = id,
    sender = sender,
    status = SupportMessageStatus.Sent,
    createdAt = createdAt,
    images = emptyList(),
)
