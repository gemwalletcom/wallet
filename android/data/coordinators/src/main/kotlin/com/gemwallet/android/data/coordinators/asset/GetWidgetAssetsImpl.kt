package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.assets.cases.GetWidgetAssets
import com.gemwallet.android.application.tokens.cases.SearchTokens
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext

class GetWidgetAssetsImpl(
    private val searchTokensCase: SearchTokens,
    private val getWalletAssets: GetWalletAssets,
) : GetWidgetAssets {

    override suspend fun invoke(): List<AssetInfo> = withContext(Dispatchers.IO) {
        runCatchingCancellable { searchTokensCase.search(WIDGET_ASSET_IDS) }
        getWalletAssets(WIDGET_ASSET_IDS).firstOrNull().orEmpty()
    }

    private companion object {
        val WIDGET_ASSET_IDS = listOf(AssetId(Chain.Bitcoin), AssetId(Chain.Ethereum), AssetId(Chain.Solana))
    }
}
