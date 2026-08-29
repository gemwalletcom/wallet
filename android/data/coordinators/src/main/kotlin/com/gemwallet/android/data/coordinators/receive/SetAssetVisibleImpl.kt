package com.gemwallet.android.data.coordinators.receive

import com.gemwallet.android.application.assets.cases.EnableAsset
import com.gemwallet.android.application.receive.cases.SetAssetVisible
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.firstOrNull

class SetAssetVisibleImpl(
    private val getSession: GetSession,
    private val enableAsset: EnableAsset,
) : SetAssetVisible {

    override suspend fun invoke(assetId: AssetId) {
        val session = getSession().firstOrNull() ?: return
        session.wallet.getAccount(assetId.chain) ?: return
        enableAsset(session.wallet.id, assetId)
    }
}
