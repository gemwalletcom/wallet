package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.application.nft.cases.RefreshNftAsset
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.NFTAssetId
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemNftService

class RefreshNftAssetImpl(
    private val getSession: GetSession,
    private val nftService: GemNftService,
) : RefreshNftAsset {

    override suspend fun invoke(assetId: NFTAssetId) {
        val wallet = getSession().firstOrNull()?.wallet ?: return
        nftService.refreshAsset(wallet.id.id, assetId.toIdentifier())
    }
}
