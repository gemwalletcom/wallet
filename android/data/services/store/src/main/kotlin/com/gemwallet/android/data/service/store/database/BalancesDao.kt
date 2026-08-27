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

    @Query("SELECT * FROM balances WHERE wallet_id = :walletId AND asset_id = :assetId")
    fun getByAsset(walletId: String, assetId: String): DbBalance?

    @Query("""
        UPDATE balances SET
            available = :available,
            available_amount = :availableAmount,
            reserved = :reserved,
            reserved_amount = :reservedAmount,
            pending_unconfirmed = :pendingUnconfirmed,
            pending_unconfirmed_amount = :pendingUnconfirmedAmount,
            total_amount = :availableAmount + frozen_amount + locked_amount + staked_amount + pending_amount + rewards_amount + earn_amount,
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
        pendingUnconfirmed: String,
        pendingUnconfirmedAmount: Double,
        isActive: Boolean,
        updatedAt: Long,
    )

    @Query("""
        UPDATE balances SET
            earn = :earn,
            earn_amount = :earnAmount,
            total_amount = available_amount + frozen_amount + locked_amount + staked_amount + pending_amount + rewards_amount + :earnAmount,
            updated_at = :updatedAt,
            is_active = :isActive
        WHERE wallet_id = :walletId AND asset_id = :assetId
    """)
    fun updateEarnBalance(
        walletId: String,
        assetId: String,
        earn: String,
        earnAmount: Double,
        isActive: Boolean,
        updatedAt: Long,
    )

    @Query("""
        UPDATE balances SET
            available = :available,
            available_amount = :availableAmount,
            total_amount = :availableAmount + frozen_amount + locked_amount + staked_amount + pending_amount + rewards_amount + earn_amount,
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
            available = :available,
            available_amount = :availableAmount,
            reserved = :reserved,
            reserved_amount = :reservedAmount,
            withdrawable = :withdrawable,
            withdrawableAmount = :withdrawableAmount,
            total_amount = :availableAmount + frozen_amount + locked_amount + staked_amount + pending_amount + rewards_amount + earn_amount,
            updated_at = :updatedAt,
            is_active = :isActive
        WHERE wallet_id = :walletId AND asset_id = :assetId
    """)
    fun updatePerpetualBalance(
        walletId: String,
        assetId: String,
        available: String,
        availableAmount: Double,
        reserved: String,
        reservedAmount: Double,
        withdrawable: String,
        withdrawableAmount: Double,
        isActive: Boolean,
        updatedAt: Long,
    )

    @Query("SELECT available_amount AS available, reserved_amount AS reserved, withdrawableAmount AS withdrawable FROM balances WHERE wallet_id = :walletId AND asset_id = :assetId")
    fun perpetualBalance(walletId: String, assetId: String): Flow<DbPerpetualBalanceProjection?>

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
            votes = COALESCE(:votes, votes),
            energy_available = COALESCE(:energyAvailable, energy_available),
            energy_total = COALESCE(:energyTotal, energy_total),
            bandwidth_available = COALESCE(:bandwidthAvailable, bandwidth_available),
            bandwidth_total = COALESCE(:bandwidthTotal, bandwidth_total),
            total_amount = available_amount + :frozenAmount + :lockedAmount + :stakedAmount + :pendingAmount + :rewardsAmount + earn_amount,
            updated_at = :updatedAt,
            is_active = :isActive
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
        votes: Long?,
        energyAvailable: Long?,
        energyTotal: Long?,
        bandwidthAvailable: Long?,
        bandwidthTotal: Long?,
        isActive: Boolean,
        updatedAt: Long,
    )
}

data class DbPerpetualBalanceProjection(
    val available: Double,
    val reserved: Double,
    val withdrawable: Double,
)
