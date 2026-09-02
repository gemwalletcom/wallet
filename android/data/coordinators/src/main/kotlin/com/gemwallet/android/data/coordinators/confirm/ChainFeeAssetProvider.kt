package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.application.confirm.cases.FeeAssetProvider
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemConfirmServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class ChainFeeAssetProvider(
    private val chain: Chain,
    private val assetStore: GemstoneAssetStore,
    private val getCurrentWalletId: GetCurrentWalletId,
    private val confirmService: GemConfirmServiceInterface,
) : FeeAssetProvider {

    override fun getFeeAssets(): Flow<List<AssetInfo>> = getCurrentWalletId().flatMapLatest { walletId ->
        combine(
            assetStore.observeAssetsInfoByChain(walletId.id, chain),
            assetStore.observeHiddenAssetsInfoByChain(walletId.id, chain),
        ) { visible, hidden -> visible + hidden }
            .map { assets ->
                val selected = confirmService.feeAssets(walletId.id, chain.string).map { it.asset.toPrimitives().id.toIdentifier() }.toSet()
                assets.filter { it.asset.id.toIdentifier() in selected }
            }
            .flowOn(Dispatchers.IO)
    }
}
