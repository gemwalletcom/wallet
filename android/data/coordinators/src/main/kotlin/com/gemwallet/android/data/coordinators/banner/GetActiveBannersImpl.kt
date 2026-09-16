package com.gemwallet.android.data.coordinators.banner

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.stores.GemstoneBannerStore
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import uniffi.gemstone.GemBannerContext
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class GetActiveBannersImpl(
    private val getSession: GetSession,
    private val getAssetInfo: GetAssetInfo,
    private val bannerStore: GemstoneBannerStore,
) : GetActiveBanners {

    override fun invoke(asset: Asset): Flow<List<Banner>> = getSession()
        .flatMapLatest { session ->
            val wallet = session?.wallet
            combine(
                bannerStore.observeAssetBanners(wallet?.id?.id, asset.id),
                getAssetInfo(asset.id),
            ) { records, assetInfo ->
                bannerContext(wallet, assetInfo)
                    .visibleBanners(stored = records.map { it.toDTO().toGem() })
                    .map { it.toPrimitives() }
            }
        }
        .flowOn(Dispatchers.IO)

    private fun bannerContext(wallet: Wallet?, assetInfo: AssetInfo?) = GemBannerContext(
        wallet = wallet?.toGem(),
        asset = assetInfo?.asset?.toGem(),
        isStakeable = assetInfo?.metadata?.isStakeEnabled == true,
        hasStakeBalance = hasStakeBalance(assetInfo),
        hasAvailableBalance = (assetInfo?.balance?.balance?.available ?: BigInteger.ZERO) > BigInteger.ZERO,
        isAssetActivated = assetInfo?.balance?.isActive != false,
        assetRankScore = assetInfo?.metadata?.rankScore,
        isWalletEmpty = false,
    )

    private fun hasStakeBalance(assetInfo: AssetInfo?): Boolean {
        val balance = assetInfo?.balance ?: return false
        return balance.toGem().stakedValue(assetInfo.asset.chain.string) > BigInteger.ZERO
    }
}
