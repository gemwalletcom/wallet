package com.gemwallet.android.data.service.store.database.entities

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NodeState

@Entity(
    tableName = "nodes",
    indices = [Index("chain")],
    foreignKeys = [
        ForeignKey(DbAsset::class, ["id"], ["chain"], onDelete = ForeignKey.CASCADE, onUpdate = ForeignKey.CASCADE),
    ],
)
data class DbNode(
    @PrimaryKey val url: String,
    val status: NodeState,
    val priority: Int,
    val chain: Chain,
)
