package com.gemwallet.android.data.services.store.database.entities

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.Balance
import com.wallet.core.primitives.BalanceMetadata
import java.math.BigInteger

@Entity(
    tableName = "balances",
    primaryKeys = ["asset_id", "wallet_id"],
    indices = [Index("wallet_id")],
    foreignKeys = [
        ForeignKey(
            entity = DbAsset::class,
            parentColumns = ["id"],
            childColumns = ["asset_id"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
        ForeignKey(
            entity = DbWallet::class,
            parentColumns = ["id"],
            childColumns = ["wallet_id"],
            onDelete = ForeignKey.CASCADE,
            onUpdate = ForeignKey.CASCADE,
        ),
    ],
)
data class DbBalance(
    @ColumnInfo("asset_id") val assetId: String,
    @ColumnInfo("wallet_id") val walletId: String,

    var available: String = "0",
    @ColumnInfo("available_amount") var availableAmount: Double = 0.0,

    var frozen: String = "0",
    @ColumnInfo("frozen_amount") var frozenAmount: Double = 0.0,

    var locked: String = "0",
    @ColumnInfo("locked_amount") var lockedAmount: Double = 0.0,

    var staked: String = "0",
    @ColumnInfo("staked_amount") var stakedAmount: Double = 0.0,

    var pending: String = "0",
    @ColumnInfo("pending_amount") var pendingAmount: Double = 0.0,

    var rewards: String = "0",
    @ColumnInfo("rewards_amount") var rewardsAmount: Double = 0.0,

    var reserved: String = "0",
    @ColumnInfo("reserved_amount") var reservedAmount: Double = 0.0,

    var withdrawable: String = "0",
    var withdrawableAmount: Double = 0.0,

    @ColumnInfo("pending_unconfirmed", defaultValue = "0") var pendingUnconfirmed: String = "0",
    @ColumnInfo("pending_unconfirmed_amount", defaultValue = "0.0") var pendingUnconfirmedAmount: Double = 0.0,

    @ColumnInfo("earn", defaultValue = "0") var earn: String = "0",
    @ColumnInfo("earn_amount", defaultValue = "0.0") var earnAmount: Double = 0.0,

    @ColumnInfo("total_amount") var totalAmount: Double = 0.0,
    @ColumnInfo("is_active") var isActive: Boolean = true,
    @ColumnInfo("is_pinned") var isPinned: Boolean = false,
    @ColumnInfo("is_visible") var isVisible: Boolean = false,
    @ColumnInfo("metadata") var metadata: BalanceMetadata? = null,
    @ColumnInfo("updated_at") var updatedAt: Long?,
)

fun DbBalance.toDTO(): AssetBalance? {
    return AssetBalance(
        asset = assetId.toAssetId()?.chain?.asset() ?: return null,
        balance = Balance(
            available = available.toBigInteger(),
            frozen = frozen.toBigInteger(),
            locked = locked.toBigInteger(),
            staked = staked.toBigInteger(),
            pending = pending.toBigInteger(),
            rewards = rewards.toBigInteger(),
            reserved = reserved.toBigInteger(),
            withdrawable = withdrawable.toBigInteger(),
            pendingUnconfirmed = pendingUnconfirmed.toBigInteger(),
            earn = earn.toBigInteger(),
        ),
        isActive = isActive,
        metadata = metadata,
    )
}
