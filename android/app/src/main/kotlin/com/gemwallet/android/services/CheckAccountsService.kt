package com.gemwallet.android.services

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.operators.AddAccountsOperator
import com.gemwallet.android.cases.device.SyncSubscription
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.ext.available
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class CheckAccountsService @Inject constructor(
    private val walletsRepository: WalletsRepository,
    private val assetsRepository: AssetsRepository,
    private val addAccountsOperator: AddAccountsOperator,
    private val passwordStore: PasswordStore,
    private val syncSubscription: SyncSubscription,
) {
    suspend operator fun invoke() = withContext(Dispatchers.IO) {
        // TODO: Remove after legacy native assets with stale rank = 0 have been repaired.
        assetsRepository.updateNativeAssetRanks()

        val wallets = walletsRepository.getAll().firstOrNull() ?: emptyList()

        wallets.forEach { wallet ->
            val nativeAssets = assetsRepository.getNativeAssets(wallet)

            if (wallet.type != WalletType.Multicoin) {
                if (nativeAssets.isEmpty()) {
                    assetsRepository.invalidateDefault(wallet)
                }
                return@forEach
            }

            val accountChains = wallet.accounts.map { it.chain }.toSet()
            val newChains = Chain.available().filterNot(accountChains::contains)

            // Backfill new chains via add_accounts, skips gracefully if the v4 keystore isn't ready (e.g. migration pending) and retries next launch.
            val newAccounts = if (newChains.isNotEmpty()) {
                runCatching { addAccountsOperator(wallet, newChains, passwordStore.getPassword(wallet.id.id)) }.getOrNull()
            } else {
                null
            }
            if (newAccounts != null) {
                val newWallet = wallet.copy(accounts = wallet.accounts + newAccounts)
                walletsRepository.updateWallet(newWallet)
                walletsRepository.updateAccounts(newWallet)
                if (newAccounts.isNotEmpty()) {
                    assetsRepository.invalidateDefault(newWallet)
                }
                syncSubscription.syncSubscription(walletsRepository.getAll().firstOrNull() ?: emptyList())
                return@forEach
            }

            val nativeAssetIds = nativeAssets.map { it.id }.toSet()
            val missingNativeAssetIds = accountChains.map { AssetId(it) }.filterNot(nativeAssetIds::contains)
            if (missingNativeAssetIds.isNotEmpty()) {
                assetsRepository.invalidateDefault(wallet)
            }
        }
    }
}
