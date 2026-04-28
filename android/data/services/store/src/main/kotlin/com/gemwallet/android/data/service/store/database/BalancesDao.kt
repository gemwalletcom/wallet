package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Update
import com.gemwallet.android.data.service.store.database.entities.DbBalance

@Dao
interface BalancesDao {
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    fun insert(balance: DbBalance)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    fun insert(balance: List<DbBalance>)

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    fun insertIgnore(balance: DbBalance)

    @Update
    fun update(balance: DbBalance)

    @Query("SELECT * FROM balances WHERE wallet_id = :walletId AND asset_id = :assetId")
    fun getByAsset(walletId: String, assetId: String): DbBalance?

    @Query("""
        UPDATE balances SET
            available = :available,
            available_amount = :availableAmount,
            reserved = :reserved,
            reserved_amount = :reservedAmount,
            total_amount = :availableAmount + frozen_amount + locked_amount + staked_amount + pending_amount + rewards_amount,
            updated_at = :updatedAt,
            is_active = :isActive
        WHERE wallet_id = :walletId AND asset_id = :assetId
    """)
    fun updateCoinBalance(
        walletId: String,
        assetId: String,
        available: String,
        availableAmount: Double,
        reserved: String,
        reservedAmount: Double,
        isActive: Boolean,
        updatedAt: Long,
    )

    @Query("""
        UPDATE balances SET
            available = :available,
            available_amount = :availableAmount,
            total_amount = :availableAmount + frozen_amount + locked_amount + staked_amount + pending_amount + rewards_amount,
            updated_at = :updatedAt,
            is_active = :isActive
        WHERE wallet_id = :walletId AND asset_id = :assetId
    """)
    fun updateTokenBalance(
        walletId: String,
        assetId: String,
        available: String,
        availableAmount: Double,
        isActive: Boolean,
        updatedAt: Long,
    )

    @Query("""
        UPDATE balances SET
            staked = :staked,
            staked_amount = :stakedAmount,
            frozen = :frozen,
            frozen_amount = :frozenAmount,
            locked = :locked,
            locked_amount = :lockedAmount,
            pending = :pending,
            pending_amount = :pendingAmount,
            rewards = :rewards,
            rewards_amount = :rewardsAmount,
            votes = :votes,
            energy_available = :energyAvailable,
            energy_total = :energyTotal,
            bandwidth_available = :bandwidthAvailable,
            bandwidth_total = :bandwidthTotal,
            total_amount = available_amount + :frozenAmount + :lockedAmount + :stakedAmount + :pendingAmount + :rewardsAmount,
            updated_at = :updatedAt
        WHERE wallet_id = :walletId AND asset_id = :assetId
    """)
    fun updateStakeBalance(
        walletId: String,
        assetId: String,
        staked: String,
        stakedAmount: Double,
        frozen: String,
        frozenAmount: Double,
        locked: String,
        lockedAmount: Double,
        pending: String,
        pendingAmount: Double,
        rewards: String,
        rewardsAmount: Double,
        votes: Long,
        energyAvailable: Long,
        energyTotal: Long,
        bandwidthAvailable: Long,
        bandwidthTotal: Long,
        updatedAt: Long,
    )
}
