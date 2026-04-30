package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.blockchain.services.BalancesService
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetBalance
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.withContext

class UpdateBalances(
    private val balancesDao: BalancesDao,
    private val balancesService: BalancesService,
) {

    suspend fun updateBalances(
        walletId: String,
        account: Account,
        tokens: List<Asset>,
    ): List<AssetBalance> = withContext(Dispatchers.IO) {
        val updatedAt = System.currentTimeMillis()

        val getNative = async { balancesService.getNativeBalances(account) }
        val getStake = async { balancesService.getStakeBalances(account) }
        val getTokens = async { updateTokensBalance(walletId, account, tokens, updatedAt) }

        val native = getNative.await()
        val stake = getStake.await()
        val tokenResults = getTokens.await()

        val nativeResult = updateNativeBalance(walletId, account, updatedAt, native, stake)

        listOfNotNull(nativeResult) + tokenResults
    }

    private fun updateNativeBalance(
        walletId: String,
        account: Account,
        updatedAt: Long,
        native: AssetBalance?,
        stake: AssetBalance?,
    ): AssetBalance? {
        val assetId = account.chain.string

        if (native == null && stake == null) {
            return balancesDao.getByAsset(walletId, assetId)?.toDTO()
        }

        ensureRowExists(walletId, assetId, updatedAt)

        native?.let {
            balancesDao.updateCoinBalance(
                walletId = walletId,
                assetId = assetId,
                available = it.balance.available,
                availableAmount = it.balanceAmount.available,
                reserved = it.balance.reserved,
                reservedAmount = it.balanceAmount.reserved,
                isActive = it.isActive,
                updatedAt = updatedAt,
            )
        }

        stake?.let {
            balancesDao.updateStakeBalance(
                walletId = walletId,
                assetId = assetId,
                staked = it.balance.staked,
                stakedAmount = it.balanceAmount.staked,
                frozen = it.balance.frozen,
                frozenAmount = it.balanceAmount.frozen,
                locked = it.balance.locked,
                lockedAmount = it.balanceAmount.locked,
                pending = it.balance.pending,
                pendingAmount = it.balanceAmount.pending,
                rewards = it.balance.rewards,
                rewardsAmount = it.balanceAmount.rewards,
                votes = it.metadata?.votes?.toLong() ?: 0L,
                energyAvailable = it.metadata?.energyAvailable?.toLong() ?: 0L,
                energyTotal = it.metadata?.energyTotal?.toLong() ?: 0L,
                bandwidthAvailable = it.metadata?.bandwidthAvailable?.toLong() ?: 0L,
                bandwidthTotal = it.metadata?.bandwidthTotal?.toLong() ?: 0L,
                updatedAt = updatedAt,
            )
        }

        return balancesDao.getByAsset(walletId, assetId)?.toDTO()
    }

    private suspend fun updateTokensBalance(
        walletId: String,
        account: Account,
        tokens: List<Asset>,
        updatedAt: Long,
    ): List<AssetBalance> {
        if (tokens.none { it.id.tokenId != null }) return emptyList()
        val balances = balancesService.getTokensBalances(account, tokens)
        for (balance in balances) {
            val assetId = balance.asset.id.toIdentifier()
            ensureRowExists(walletId, assetId, updatedAt)
            balancesDao.updateTokenBalance(
                walletId = walletId,
                assetId = assetId,
                available = balance.balance.available,
                availableAmount = balance.balanceAmount.available,
                isActive = balance.isActive,
                updatedAt = updatedAt,
            )
        }
        return balances
    }

    private fun ensureRowExists(walletId: String, assetId: String, updatedAt: Long) {
        balancesDao.insertIgnore(
            DbBalance(assetId = assetId, walletId = walletId, isVisible = false, updatedAt = updatedAt)
        )
    }
}
