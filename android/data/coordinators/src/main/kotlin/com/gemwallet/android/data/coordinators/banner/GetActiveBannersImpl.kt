package com.gemwallet.android.data.coordinators.banner

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.stores.GemstoneBannerStore
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemBannerContext
import uniffi.gemstone.GemBannerItem
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class GetActiveBannersImpl(
    private val getSession: GetSession,
    private val getAssetInfo: GetAssetInfo,
    private val getWalletAssets: GetWalletAssets,
    private val bannerStore: GemstoneBannerStore,
) : GetActiveBanners {

    override fun invoke(asset: Asset?, isGlobal: Boolean): Flow<List<Banner>> = getSession()
        .flatMapLatest { session ->
            val wallet = session?.wallet
            val sceneWallet = wallet.takeUnless { isGlobal }
            val stored = when {
                asset != null -> bannerStore.observeAssetBanners(wallet?.id?.id, asset.id)
                wallet != null -> bannerStore.observeWalletBanners(wallet.id.id, listOf(BannerEvent.AccountBlockedMultiSignature, BannerEvent.Onboarding))
                else -> flowOf(emptyList())
            }
            val assetInfo = asset?.id?.let { getAssetInfo(it) } ?: flowOf(null)
            val isWalletEmpty = if (asset == null) {
                getWalletAssets().map { items -> items.all { it.balance.totalAmount == 0.0 } }
            } else {
                flowOf(false)
            }
            combine(stored, assetInfo, isWalletEmpty) { records, assetInfo, isWalletEmpty ->
                val banners = records.map { it.toDTO() }
                bannerContext(wallet, assetInfo, isWalletEmpty).visibleBanners(
                    stored = banners.map { GemBannerItem(event = it.event.toGem(), state = it.state.toGem(), assetId = it.asset?.id?.toIdentifier()) },
                ).map { item ->
                    val event = item.event.toPrimitives()
                    banners.firstOrNull { it.event == event && it.asset?.id?.toIdentifier() == item.assetId }
                        ?: Banner(walletId = sceneWallet?.id, asset = assetInfo?.asset, state = item.state.toPrimitives(), event = event)
                }
            }
        }
        .flowOn(Dispatchers.IO)

    private fun bannerContext(wallet: Wallet?, assetInfo: AssetInfo?, isWalletEmpty: Boolean) = GemBannerContext(
        wallet = wallet?.toGem(),
        assetId = assetInfo?.asset?.id?.toIdentifier(),
        isStakeable = assetInfo?.metadata?.isStakeEnabled == true,
        hasStakeBalance = hasStakeBalance(assetInfo),
        hasAvailableBalance = (assetInfo?.balance?.balance?.available ?: BigInteger.ZERO) > BigInteger.ZERO,
        isAssetActivated = assetInfo?.balance?.isActive != false,
        assetRankScore = assetInfo?.metadata?.rankScore,
        isWalletEmpty = isWalletEmpty,
    )

    private fun hasStakeBalance(assetInfo: AssetInfo?): Boolean {
        val balance = assetInfo?.balance ?: return false
        return balance.toGem().stakedValue(assetInfo.asset.chain.string) > BigInteger.ZERO
    }
}
