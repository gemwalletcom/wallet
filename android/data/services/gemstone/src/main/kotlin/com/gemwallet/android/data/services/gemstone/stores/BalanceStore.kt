package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.AssetsDao
import uniffi.gemstone.GemAssetBalance
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemBalanceStore
import uniffi.gemstone.GemBalanceUpdate
import uniffi.gemstone.GemBalanceUpdateType

class GemstoneBalanceStore(
    private val balancesDao: BalancesDao,
    private val assetsDao: AssetsDao,
    private val transactionRunner: StoreTransactionRunner,
) : GemBalanceStore {

    override suspend fun getAvailableBalances(walletId: String, assetIds: List<String>): List<GemAssetBalance> = withContext(Dispatchers.IO) {
        assetIds.mapNotNull { balancesDao.getByAsset(walletId, it)?.toGemAssetBalance() }
    }

    override suspend fun getEnabledAssetIds(walletId: String): List<String> = balancesDao.getEnabledAssetIds(walletId)

    override suspend fun setAssetsEnabled(walletId: String, assetIds: List<String>, enabled: Boolean) =
        assetsDao.setWalletAssetsVisibility(walletId, assetIds, enabled)

    override suspend fun setAssetPinned(walletId: String, assetId: String, pinned: Boolean) {
        val balance = assetsDao.getBalance(walletId, assetId) ?: return
        assetsDao.setBalanceConfig(walletId, assetId, isPinned = pinned, isVisible = balance.isVisible, listPosition = balance.listPosition)
    }

    override suspend fun updateBalances(walletId: String, updates: List<GemBalanceUpdate>) = transactionRunner.run {
        val updatedAt = System.currentTimeMillis()
        for (update in updates) {
            when (val type = update.updateType) {
                is GemBalanceUpdateType.Coin -> balancesDao.updateCoinBalance(
                    walletId = walletId,
                    assetId = update.assetId,
                    available = type.available.value.toString(),
                    availableAmount = type.available.amount,
                    frozen = type.frozen.value.toString(),
                    frozenAmount = type.frozen.amount,
                    reserved = type.reserved.value.toString(),
                    reservedAmount = type.reserved.amount,
                    pendingUnconfirmed = type.pendingUnconfirmed.value.toString(),
                    pendingUnconfirmedAmount = type.pendingUnconfirmed.amount,
                    isActive = update.isActive,
                    updatedAt = updatedAt,
                )
                is GemBalanceUpdateType.Token -> balancesDao.updateTokenBalance(
                    walletId = walletId,
                    assetId = update.assetId,
                    available = type.available.value.toString(),
                    availableAmount = type.available.amount,
                    isActive = update.isActive,
                    updatedAt = updatedAt,
                )
                is GemBalanceUpdateType.Stake -> {
                    val metadata = type.metadata?.toPrimitives()
                    balancesDao.updateStakeBalance(
                        walletId = walletId,
                        assetId = update.assetId,
                        staked = type.staked.value.toString(),
                        stakedAmount = type.staked.amount,
                        frozen = type.frozen.value.toString(),
                        frozenAmount = type.frozen.amount,
                        locked = type.locked.value.toString(),
                        lockedAmount = type.locked.amount,
                        pending = type.pending.value.toString(),
                        pendingAmount = type.pending.amount,
                        rewards = type.rewards.value.toString(),
                        rewardsAmount = type.rewards.amount,
                        votes = metadata?.votes?.toLong(),
                        energyAvailable = metadata?.energyAvailable?.toLong(),
                        energyTotal = metadata?.energyTotal?.toLong(),
                        bandwidthAvailable = metadata?.bandwidthAvailable?.toLong(),
                        bandwidthTotal = metadata?.bandwidthTotal?.toLong(),
                        isActive = update.isActive,
                        updatedAt = updatedAt,
                    )
                }
                is GemBalanceUpdateType.Perpetual -> balancesDao.updatePerpetualBalance(
                    walletId = walletId,
                    assetId = update.assetId,
                    available = type.available.value.toString(),
                    availableAmount = type.available.amount,
                    reserved = type.reserved.value.toString(),
                    reservedAmount = type.reserved.amount,
                    withdrawable = type.withdrawable.value.toString(),
                    withdrawableAmount = type.withdrawable.amount,
                    isActive = update.isActive,
                    updatedAt = updatedAt,
                )
                is GemBalanceUpdateType.Earn -> balancesDao.updateEarnBalance(
                    walletId = walletId,
                    assetId = update.assetId,
                    earn = type.balance.value.toString(),
                    earnAmount = type.balance.amount,
                    isActive = update.isActive,
                    updatedAt = updatedAt,
                )
            }
        }
    }
}
