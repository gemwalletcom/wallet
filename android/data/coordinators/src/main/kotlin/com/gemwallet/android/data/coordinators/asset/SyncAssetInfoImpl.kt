package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.SyncAssetInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.stream.StreamSubscriptionService
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext

class SyncAssetInfoImpl(
    private val gemApiClient: GemApiClient,
    private val assetsRepository: AssetsRepository,
    private val streamSubscriptionService: StreamSubscriptionService,
) : SyncAssetInfo {

    override suspend fun syncAssetInfo(assetId: AssetId, wallet: Wallet): Unit = withContext(Dispatchers.IO) {
        val account = wallet.getAccount(assetId) ?: return@withContext

        streamSubscriptionService.addAssetIds(listOf(assetId))

        coroutineScope {
            async {
                ensureWalletAsset(
                    walletId = wallet.id,
                    accountAddress = account.address,
                    assetId = assetId,
                )
            }
            async { assetsRepository.updateBalances(assetId) }
            async {
                val assetFull = loadAssetMetadata(assetId) ?: return@async
                assetsRepository.updateAssetMetadata(assetFull)
            }
        }
    }

    private suspend fun ensureWalletAsset(
        walletId: String,
        accountAddress: String,
        assetId: AssetId,
    ) = assetsRepository.getAssetInfo(assetId).firstOrNull()
        ?: assetsRepository.getTokenInfo(assetId).firstOrNull()?.also { asset ->
            assetsRepository.linkAssetToWallet(
                walletId = walletId,
                accountAddress = accountAddress,
                assetId = asset.asset.id,
                visible = asset.metadata?.isBalanceEnabled ?: true,
            )
        }

    private suspend fun loadAssetMetadata(assetId: AssetId) =
        runCatching { gemApiClient.getAsset(assetId.toIdentifier()) }.getOrNull()
}
