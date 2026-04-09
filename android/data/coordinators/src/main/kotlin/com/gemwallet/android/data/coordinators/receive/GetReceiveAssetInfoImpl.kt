package com.gemwallet.android.data.coordinators.receive

import com.gemwallet.android.application.receive.coordinators.GetReceiveAssetInfo
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach

@OptIn(ExperimentalCoroutinesApi::class)
class GetReceiveAssetInfoImpl(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
) : GetReceiveAssetInfo {

    override fun invoke(assetId: AssetId): Flow<AssetInfo?> {
        return sessionRepository.session()
            .filterNotNull()
            .onEach { session ->
                assetsRepository.addRecentReceive(assetId, session.wallet.id)
            }
            .flatMapLatest { session ->
                assetsRepository.getTokenInfo(assetId).map { info ->
                    if (info?.owner == null) {
                        info?.copy(owner = session.wallet.getAccount(info.asset.chain))
                    } else {
                        info
                    }
                }
            }
    }
}
