package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetBuyAssetInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
class GetBuyAssetInfoImpl(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
) : GetBuyAssetInfo {

    override fun invoke(assetId: AssetId): Flow<AssetInfo?> {
        return combine(sessionRepository.session(), assetsRepository.getTokenInfo(assetId)) { session, assetInfo ->
            if (assetInfo?.owner == null && session != null) {
                assetInfo?.copy(owner = session.wallet.getAccount(assetInfo.asset.chain))
            } else {
                assetInfo
            }
        }
    }
}
