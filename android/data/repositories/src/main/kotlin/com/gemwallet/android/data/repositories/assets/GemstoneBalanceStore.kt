package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.BalanceMetadata
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemBalanceStore
import uniffi.gemstone.GemBalanceUpdate
import uniffi.gemstone.GemBalanceUpdateType

class GemstoneBalanceStore(
    private val balancesDao: BalancesDao,
) : GemBalanceStore {

    override suspend fun updateBalances(walletId: String, updates: List<GemBalanceUpdate>) = withContext(Dispatchers.IO) {
        val updatedAt = System.currentTimeMillis()
        for (update in updates) {
            balancesDao.insertIgnore(DbBalance(assetId = update.assetId, walletId = walletId, isVisible = false, updatedAt = updatedAt))
            when (val type = update.updateType) {
                is GemBalanceUpdateType.Coin -> balancesDao.updateCoinBalance(
                    walletId = walletId,
                    assetId = update.assetId,
                    available = type.available.value,
                    availableAmount = type.available.amount,
                    reserved = type.reserved.value,
                    reservedAmount = type.reserved.amount,
                    isActive = update.isActive,
                    updatedAt = updatedAt,
                )
                is GemBalanceUpdateType.Token -> balancesDao.updateTokenBalance(
                    walletId = walletId,
                    assetId = update.assetId,
                    available = type.available.value,
                    availableAmount = type.available.amount,
                    isActive = update.isActive,
                    updatedAt = updatedAt,
                )
                is GemBalanceUpdateType.Stake -> {
                    val metadata = type.metadata?.decodeJson<BalanceMetadata>()
                    balancesDao.updateStakeBalance(
                        walletId = walletId,
                        assetId = update.assetId,
                        staked = type.staked.value,
                        stakedAmount = type.staked.amount,
                        frozen = type.frozen.value,
                        frozenAmount = type.frozen.amount,
                        locked = type.locked.value,
                        lockedAmount = type.locked.amount,
                        pending = type.pending.value,
                        pendingAmount = type.pending.amount,
                        rewards = type.rewards.value,
                        rewardsAmount = type.rewards.amount,
                        votes = metadata?.votes?.toLong() ?: 0L,
                        energyAvailable = metadata?.energyAvailable?.toLong() ?: 0L,
                        energyTotal = metadata?.energyTotal?.toLong() ?: 0L,
                        bandwidthAvailable = metadata?.bandwidthAvailable?.toLong() ?: 0L,
                        bandwidthTotal = metadata?.bandwidthTotal?.toLong() ?: 0L,
                        updatedAt = updatedAt,
                    )
                }
                is GemBalanceUpdateType.Earn -> Unit
            }
        }
    }
}
