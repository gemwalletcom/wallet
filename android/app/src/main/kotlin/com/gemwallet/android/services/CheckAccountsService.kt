package com.gemwallet.android.services

import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.domains.asset.defaultAssetRank
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Wallet
import javax.inject.Inject
import javax.inject.Singleton
import uniffi.gemstone.GemAppStartService

@Singleton
class CheckAccountsService @Inject constructor(
    private val walletsRepository: WalletsRepository,
    private val assetsRepository: AssetsRepository,
    private val appStartService: GemAppStartService,
) {
    suspend operator fun invoke() = withContext(Dispatchers.IO) {
        val updatedWallets = appStartService.setupWallets()
            .map { it.decodeJson<Wallet>() }
            .associateBy { it.id }
        assetsRepository.updateNativeAssetRanks()
        val wallets = walletsRepository.getAll().firstOrNull() ?: emptyList()

        wallets.forEach { wallet ->
            val nativeAssets = assetsRepository.getNativeAssets(wallet)
            val accountChains = wallet.accounts.map { it.chain }.toSet()
            val expectedNativeAssetIds = accountChains.filter { it.defaultAssetRank >= 0 }.map(::AssetId)

            if (wallet.type != WalletType.Multicoin) {
                if (expectedNativeAssetIds.isNotEmpty() && nativeAssets.isEmpty()) {
                    assetsRepository.invalidateDefault(wallet)
                }
                assetsRepository.ensureDefaultAssets(wallet)
                return@forEach
            }

            val updatedWallet = updatedWallets[wallet.id]
            if (updatedWallet != null) {
                assetsRepository.invalidateDefault(updatedWallet)
                assetsRepository.ensureDefaultAssets(updatedWallet)
                return@forEach
            }

            val nativeAssetIds = nativeAssets.map { it.id }.toSet()
            val missingNativeAssetIds = expectedNativeAssetIds.filterNot(nativeAssetIds::contains)
            if (missingNativeAssetIds.isNotEmpty()) {
                assetsRepository.invalidateDefault(wallet)
            }
            assetsRepository.ensureDefaultAssets(wallet)
        }
    }
}
