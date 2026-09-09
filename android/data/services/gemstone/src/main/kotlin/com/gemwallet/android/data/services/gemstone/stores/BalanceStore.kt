package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.AssetsDao
import uniffi.gemstone.GemAssetBalance
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemBalanceStore
import uniffi.gemstone.GemBalanceRecord

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

    override suspend fun updateBalances(walletId: String, balances: List<GemBalanceRecord>) = transactionRunner.run {
        val updatedAt = System.currentTimeMillis()
        for (balance in balances) {
            val metadata = balance.metadata?.toPrimitives()
            balancesDao.updateBalance(
                walletId = walletId,
                assetId = balance.assetId,
                available = balance.available.value.toString(),
                availableAmount = balance.available.amount,
                frozen = balance.frozen.value.toString(),
                frozenAmount = balance.frozen.amount,
                locked = balance.locked.value.toString(),
                lockedAmount = balance.locked.amount,
                staked = balance.staked.value.toString(),
                stakedAmount = balance.staked.amount,
                pending = balance.pending.value.toString(),
                pendingAmount = balance.pending.amount,
                pendingUnconfirmed = balance.pendingUnconfirmed.value.toString(),
                pendingUnconfirmedAmount = balance.pendingUnconfirmed.amount,
                rewards = balance.rewards.value.toString(),
                rewardsAmount = balance.rewards.amount,
                reserved = balance.reserved.value.toString(),
                reservedAmount = balance.reserved.amount,
                withdrawable = balance.withdrawable.value.toString(),
                withdrawableAmount = balance.withdrawable.amount,
                earn = balance.earn.value.toString(),
                earnAmount = balance.earn.amount,
                votes = metadata?.votes?.toLong() ?: 0L,
                energyAvailable = metadata?.energyAvailable?.toLong() ?: 0L,
                energyTotal = metadata?.energyTotal?.toLong() ?: 0L,
                bandwidthAvailable = metadata?.bandwidthAvailable?.toLong() ?: 0L,
                bandwidthTotal = metadata?.bandwidthTotal?.toLong() ?: 0L,
                isActive = balance.isActive,
                updatedAt = updatedAt,
            )
        }
    }
}
