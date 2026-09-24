package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.StoreConverters
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.ext.toPrimitives
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemAssetConfiguration
import uniffi.gemstone.GemBalanceRecord
import uniffi.gemstone.GemBalanceStore

class GemstoneBalanceStore(private val balancesDao: BalancesDao, private val assetsDao: AssetsDao, private val transactionRunner: StoreTransactionRunner) : GemBalanceStore {

    private val converters = StoreConverters()

    override suspend fun getAvailableBalances(walletId: String, assetIds: List<String>): List<GemAssetBalance> = withContext(Dispatchers.IO) {
        balancesDao.getByAssets(walletId, assetIds).map { it.toGemAssetBalance() }
    }

    override suspend fun getBalanceAssetIds(walletId: String, assetIds: List<String>): List<String> = balancesDao.getAssetIds(walletId, assetIds)

    override suspend fun getEnabledAssetIds(walletId: String): List<String> = balancesDao.getEnabledAssetIds(walletId)

    override suspend fun setAssetConfiguration(walletId: String, assetIds: List<String>, configuration: GemAssetConfiguration) =
        assetsDao.setAssetConfiguration(walletId, assetIds, isVisible = configuration.isEnabled, isPinned = configuration.isPinned)

    override suspend fun updateBalances(walletId: String, balances: List<GemBalanceRecord>) = transactionRunner.run {
        val updatedAt = System.currentTimeMillis()
        for (balance in balances) {
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
                metadata = converters.fromBalanceMetadata(balance.metadata?.toPrimitives()),
                isActive = balance.isActive,
                updatedAt = updatedAt,
            )
        }
    }
}
