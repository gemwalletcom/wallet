package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.BalanceMetadata
import uniffi.gemstone.GemBalanceStore
import uniffi.gemstone.GemBalanceUpdate
import uniffi.gemstone.GemBalanceUpdateType

class GemstoneBalanceStore(
    private val balancesDao: BalancesDao,
    private val assetsDao: AssetsDao,
    private val transactionRunner: StoreTransactionRunner,
) : GemBalanceStore {

    override suspend fun getEnabledAssetIds(walletId: String, assetIds: List<String>): List<String> =
        balancesDao.getVisibleAssetIds(walletId, assetIds)

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
