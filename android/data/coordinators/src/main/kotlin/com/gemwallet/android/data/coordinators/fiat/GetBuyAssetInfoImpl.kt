package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.cases.GetBuyAssetInfo
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.model.AssetData
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine

class GetBuyAssetInfoImpl(
    private val getSession: GetSession,
    private val getAssetTokenInfo: GetAssetTokenInfo,
) : GetBuyAssetInfo {

    override fun invoke(assetId: AssetId): Flow<AssetData?> {
        return combine(getSession(), getAssetTokenInfo(assetId)) { session, assetInfo ->
            val wallet = session?.wallet ?: return@combine null
            val info = assetInfo ?: return@combine null
            val account = wallet.getAccount(assetId) ?: return@combine null

            AssetData.from(info, wallet, account)
        }
    }
}
