package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemBalanceService

class EnableAssetImpl(
    private val sessionRepository: SessionRepository,
    private val balanceService: GemBalanceService,
) : EnableAsset {

    override suspend fun invoke(walletId: WalletId, assetId: AssetId, enabled: Boolean) = invoke(walletId, listOf(assetId), enabled)

    override suspend fun invoke(walletId: WalletId, assetIds: List<AssetId>, enabled: Boolean) {
        balanceService.enableAssets(walletId.id, assetIds.map { it.toIdentifier() }, enabled, sessionRepository.getCurrentCurrency().toJson())
    }
}
