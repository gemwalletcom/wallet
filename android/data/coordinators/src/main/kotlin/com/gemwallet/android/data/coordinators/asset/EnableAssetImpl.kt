package com.gemwallet.android.data.coordinators.asset

import android.util.Log
import com.gemwallet.android.application.assets.cases.EnableAsset
import com.gemwallet.android.ext.runCatchingCancellable
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

    override suspend fun invoke(walletId: WalletId, assetIds: List<AssetId>, enabled: Boolean) {
        val identifiers = assetIds.map { it.toIdentifier() }
        withContext(Dispatchers.IO) {
            runCatchingCancellable { balanceService.setAssetsEnabled(walletId.id, identifiers, enabled) }
                .onFailure { Log.e(TAG, "setting assets enabled=$enabled failed for $identifiers", it) }
        }
    }

    private companion object {
        const val TAG = "EnableAsset"
    }
}
