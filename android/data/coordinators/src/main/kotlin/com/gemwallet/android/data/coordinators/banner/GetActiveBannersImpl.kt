package com.gemwallet.android.data.coordinators.banner

import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.stores.GemstoneBannerStore
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.toGem
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
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
    private val getActiveAssetsInfo: GetActiveAssetsInfo,
    private val bannerStore: GemstoneBannerStore,
) : GetActiveBanners {

    override fun invoke(asset: Asset?, isGlobal: Boolean): Flow<List<Banner>> = getSession()
        .flatMapLatest { session ->
            val wallet = session?.wallet
            val sceneWallet = wallet.takeUnless { isGlobal }
            val stored = when {
                asset != null -> bannerStore.observeAssetBanners(wallet?.id?.id, asset.id.toIdentifier())
                wallet != null -> bannerStore.observeWalletBanners(wallet.id.id, listOf(BannerEvent.AccountBlockedMultiSignature, BannerEvent.Onboarding))
                else -> flowOf(emptyList())
            }
            val assetInfo = asset?.id?.let { getAssetInfo(it) } ?: flowOf(null)
            val isWalletEmpty = if (asset == null) {
                getActiveAssetsInfo.getAssetsInfo(hideBalance = false).map { items -> items.all { it.isZeroBalance } }
            } else {
                flowOf(false)
            }
            combine(stored, assetInfo, isWalletEmpty) { records, assetInfo, isWalletEmpty ->
                val banners = records.map { it.toDTO(asset) }.distinctBy { it.event }
                bannerContext(wallet, assetInfo, isWalletEmpty).visibleBanners(
                    stored = banners.map { GemBannerItem(event = it.event.toJson(), state = it.state.toJson()) },
                ).map { item ->
                    val event = item.event.decodeJson<BannerEvent>()
                    banners.firstOrNull { it.event == event }
                        ?: Banner(walletId = sceneWallet?.id, asset = assetInfo?.asset, state = item.state.decodeJson(), event = event)
                }
            }
        }
        .flowOn(Dispatchers.IO)

    private fun bannerContext(wallet: Wallet?, assetInfo: AssetInfo?, isWalletEmpty: Boolean) = GemBannerContext(
        wallet = wallet?.toJson(),
        hasAsset = assetInfo != null,
        isStakeable = assetInfo?.metadata?.isStakeEnabled == true,
        hasStakeBalance = hasStakeBalance(assetInfo),
        hasAvailableBalance = (assetInfo?.balance?.balance?.available?.toBigIntegerOrNull() ?: BigInteger.ZERO) > BigInteger.ZERO,
        isAssetActivated = assetInfo?.balance?.isActive != false,
        assetRankScore = assetInfo?.metadata?.rankScore,
        isWalletEmpty = isWalletEmpty,
    )

    private fun hasStakeBalance(assetInfo: AssetInfo?): Boolean {
        val balance = assetInfo?.balance ?: return false
        return balance.toGem().stakedValue(assetInfo.asset.chain.string) > BigInteger.ZERO
    }
}
