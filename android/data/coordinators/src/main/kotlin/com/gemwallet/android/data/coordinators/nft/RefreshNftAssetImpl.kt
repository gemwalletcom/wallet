package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.application.nft.cases.RefreshNftAsset
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.NFTAssetId
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemNftService

class RefreshNftAssetImpl(
    private val sessionRepository: SessionRepository,
    private val nftService: GemNftService,
) : RefreshNftAsset {

    override suspend fun invoke(assetId: NFTAssetId) {
        val wallet = sessionRepository.session().firstOrNull()?.wallet ?: return
        nftService.refreshAsset(wallet.id.id, assetId.toIdentifier())
    }
}
