package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.BalancesDao
import kotlinx.coroutines.flow.first
import com.gemwallet.android.data.service.store.database.AssetsDao
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
    private val assetsDao: AssetsDao,
) : GemBalanceStore {

    override suspend fun getEnabledAssetIds(walletId: String, assetIds: List<String>): List<String> =
        assetsDao.getAssetsInfo(walletId, assetIds).first().filter { it.visible == true }.map { it.id }

    override suspend fun setEnabled(walletId: String, assetIds: List<String>, enabled: Boolean) {
        assetIds.forEach { assetsDao.setWalletAssetVisibility(walletId, it, enabled) }
    }

    override suspend fun setPinned(walletId: String, assetId: String, pinned: Boolean) {
        val balance = assetsDao.getBalance(walletId, assetId) ?: return
        assetsDao.setBalanceConfig(walletId, assetId, isPinned = pinned, isVisible = balance.isVisible, listPosition = balance.listPosition)
    }

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
                    pendingUnconfirmed = type.pendingUnconfirmed.value,
                    pendingUnconfirmedAmount = type.pendingUnconfirmed.amount,
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
                is GemBalanceUpdateType.Perpetual -> balancesDao.updatePerpetualBalance(
                    walletId = walletId,
                    assetId = update.assetId,
                    available = type.available.value,
                    availableAmount = type.available.amount,
                    reserved = type.reserved.value,
                    reservedAmount = type.reserved.amount,
                    withdrawable = type.withdrawable.value,
                    withdrawableAmount = type.withdrawable.amount,
                    isActive = update.isActive,
                    updatedAt = updatedAt,
                )
                is GemBalanceUpdateType.Earn -> balancesDao.updateEarnBalance(
                    walletId = walletId,
                    assetId = update.assetId,
                    earn = type.balance.value,
                    earnAmount = type.balance.amount,
                    isActive = update.isActive,
                    updatedAt = updatedAt,
                )
            }
        }
    }
}
