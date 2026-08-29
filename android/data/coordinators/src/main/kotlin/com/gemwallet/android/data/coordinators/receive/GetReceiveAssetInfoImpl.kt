package com.gemwallet.android.data.coordinators.receive

import com.gemwallet.android.application.receive.cases.GetReceiveAssetInfo
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
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

@OptIn(ExperimentalCoroutinesApi::class)
class GetReceiveAssetInfoImpl(
    private val sessionRepository: SessionRepository,
    private val getAssetTokenInfo: GetAssetTokenInfo,
) : GetReceiveAssetInfo {

    override fun invoke(assetId: AssetId): Flow<AssetInfo?> {
        return sessionRepository.session()
            .filterNotNull()
            .flatMapLatest { session ->
                getAssetTokenInfo(assetId).map { info ->
                    if (info?.owner == null) {
                        info?.copy(owner = session.wallet.getAccount(info.asset.chain))
                    } else {
                        info
                    }
                }
            }
    }
}
