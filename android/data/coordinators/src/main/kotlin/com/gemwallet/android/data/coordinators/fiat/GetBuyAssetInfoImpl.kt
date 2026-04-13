package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetBuyAssetInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.model.AssetData
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine

class GetBuyAssetInfoImpl(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
) : GetBuyAssetInfo {

    override fun invoke(assetId: AssetId): Flow<AssetData?> {
        return combine(sessionRepository.session(), assetsRepository.getTokenInfo(assetId)) { session, assetInfo ->
            val wallet = session?.wallet ?: return@combine null
            val info = assetInfo ?: return@combine null
            val account = wallet.getAccount(assetId) ?: return@combine null

            AssetData.from(info, wallet, account)
        }
    }
}
