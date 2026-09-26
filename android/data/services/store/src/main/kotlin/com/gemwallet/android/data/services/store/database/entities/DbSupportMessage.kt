package com.gemwallet.android.data.services.store.database.entities

import androidx.room.Entity
import androidx.room.Index
import com.wallet.core.primitives.SupportMessageImage
import com.wallet.core.primitives.SupportMessageSender
import com.wallet.core.primitives.SupportMessageStatus

@Entity(
    tableName = "support_messages",
    primaryKeys = ["id"],
    indices = [Index("createdAt")],
)
data class DbSupportMessage(val id: String, val content: String, val sender: SupportMessageSender, val status: SupportMessageStatus, val createdAt: Long, val images: List<SupportMessageImage>)
