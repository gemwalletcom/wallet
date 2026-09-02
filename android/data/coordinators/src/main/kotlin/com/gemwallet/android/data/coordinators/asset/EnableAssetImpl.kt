package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.EnableAsset
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemBalanceService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class EnableAssetImpl(
    private val balanceService: GemBalanceService,
) : EnableAsset {

    override suspend fun invoke(walletId: WalletId, assetId: AssetId, enabled: Boolean) = invoke(walletId, listOf(assetId), enabled)

    override suspend fun invoke(walletId: WalletId, assetIds: List<AssetId>, enabled: Boolean) = withContext(Dispatchers.IO) {
        balanceService.setAssetsEnabled(walletId.id, assetIds.map { it.toIdentifier() }, enabled)
    }
}
