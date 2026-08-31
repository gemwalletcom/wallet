package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.HideAsset
import com.gemwallet.android.application.assets.cases.EnableAsset
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.AssetId

class HideAssetImpl(
    private val getSession: GetSession,
    private val enableAsset: EnableAsset,
) : HideAsset {

    override suspend fun invoke(assetId: AssetId) {
        val session = getSession().value ?: return
        session.wallet.getAccount(assetId.chain) ?: return
        enableAsset(session.wallet.id, assetId, enabled = false)
    }
}
