package com.gemwallet.android.data.service.store.database.entities

import androidx.room.Entity
import androidx.room.Index
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageImage
import com.wallet.core.primitives.SupportMessageSender
import com.wallet.core.primitives.SupportMessageStatus

@Entity(
    tableName = "support_messages",
    primaryKeys = ["id"],
    indices = [Index("createdAt")],
)
data class DbSupportMessage(
    val id: String,
    val content: String,
    val sender: SupportMessageSender,
    val status: SupportMessageStatus,
    val createdAt: Long,
    val images: List<SupportMessageImage>,
)

fun DbSupportMessage.toModel(): SupportMessage = SupportMessage(
    id = id,
    content = content,
    sender = sender,
    status = status,
    createdAt = createdAt,
    images = images,
)

fun SupportMessage.toRecord(): DbSupportMessage = DbSupportMessage(
    id = id,
    content = content,
    sender = sender,
    status = status,
    createdAt = createdAt,
    images = images,
)
