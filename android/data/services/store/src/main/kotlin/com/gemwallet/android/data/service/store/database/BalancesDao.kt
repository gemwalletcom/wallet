package com.gemwallet.android.data.service.store.database

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Update
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import kotlinx.coroutines.flow.Flow

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

    @Query("DELETE FROM balances WHERE asset_id = :assetId")
    fun deleteByAssetId(assetId: String)

    @Query("SELECT * FROM balances WHERE wallet_id = :walletId AND asset_id IN (:assetIds)")
    fun getByAssets(walletId: String, assetIds: List<String>): List<DbBalance>

    @Query("SELECT asset_id FROM balances WHERE wallet_id = :walletId AND is_visible != 0")
    suspend fun getEnabledAssetIds(walletId: String): List<String>

    @Query("""
        UPDATE balances SET
            available = :available,
            available_amount = :availableAmount,
            frozen = :frozen,
            frozen_amount = :frozenAmount,
            locked = :locked,
            locked_amount = :lockedAmount,
            staked = :staked,
            staked_amount = :stakedAmount,
            pending = :pending,
            pending_amount = :pendingAmount,
            pending_unconfirmed = :pendingUnconfirmed,
            pending_unconfirmed_amount = :pendingUnconfirmedAmount,
            rewards = :rewards,
            rewards_amount = :rewardsAmount,
            reserved = :reserved,
            reserved_amount = :reservedAmount,
            withdrawable = :withdrawable,
            withdrawableAmount = :withdrawableAmount,
            earn = :earn,
            earn_amount = :earnAmount,
            total_amount = :availableAmount + :frozenAmount + :lockedAmount + :stakedAmount + :pendingAmount + :rewardsAmount + :earnAmount,
            votes = :votes,
            energy_available = :energyAvailable,
            energy_total = :energyTotal,
            bandwidth_available = :bandwidthAvailable,
            bandwidth_total = :bandwidthTotal,
            updated_at = :updatedAt,
            is_active = :isActive
        WHERE wallet_id = :walletId AND asset_id = :assetId
    """)
    fun updateBalance(
        walletId: String,
        assetId: String,
        available: String,
        availableAmount: Double,
        frozen: String,
        frozenAmount: Double,
        locked: String,
        lockedAmount: Double,
        staked: String,
        stakedAmount: Double,
        pending: String,
        pendingAmount: Double,
        pendingUnconfirmed: String,
        pendingUnconfirmedAmount: Double,
        rewards: String,
        rewardsAmount: Double,
        reserved: String,
        reservedAmount: Double,
        withdrawable: String,
        withdrawableAmount: Double,
        earn: String,
        earnAmount: Double,
        votes: Long,
        energyAvailable: Long,
        energyTotal: Long,
        bandwidthAvailable: Long,
        bandwidthTotal: Long,
        isActive: Boolean,
        updatedAt: Long,
    )

    @Query("SELECT available_amount AS available, reserved_amount AS reserved, withdrawableAmount AS withdrawable FROM balances WHERE wallet_id = :walletId AND asset_id = :assetId")
    fun perpetualBalance(walletId: String, assetId: String): Flow<DbPerpetualBalanceProjection?>
}

data class DbPerpetualBalanceProjection(
    val available: Double,
    val reserved: Double,
    val withdrawable: Double,
)
