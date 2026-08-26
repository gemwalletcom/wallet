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

    override suspend fun invoke(walletId: WalletId, assetId: AssetId) = invoke(walletId, listOf(assetId))

    override suspend fun invoke(walletId: WalletId, assetIds: List<AssetId>) {
        balanceService.enableAssets(walletId.id, assetIds.map { it.toIdentifier() }, true, sessionRepository.getCurrentCurrency().toJson())
    }
}
