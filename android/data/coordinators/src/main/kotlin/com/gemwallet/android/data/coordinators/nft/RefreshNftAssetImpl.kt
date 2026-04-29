package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.cases.nft.RefreshNftAsset
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.firstOrNull
import com.gemwallet.android.application.nft.coordinators.RefreshNftAsset as RefreshNftAssetCoordinator

class RefreshNftAssetImpl(
    private val sessionRepository: SessionRepository,
    private val refreshNftAsset: RefreshNftAsset,
) : RefreshNftAssetCoordinator {

    override suspend fun invoke(assetId: AssetId) {
        val wallet = sessionRepository.session().firstOrNull()?.wallet ?: return
        refreshNftAsset.refreshNftAsset(wallet, assetId)
    }
}
