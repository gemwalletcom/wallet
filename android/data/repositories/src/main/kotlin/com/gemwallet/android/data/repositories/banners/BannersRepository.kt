package com.gemwallet.android.data.repositories.banners

import com.gemwallet.android.cases.banners.BannerActionCase
import com.gemwallet.android.cases.banners.GetBannersCase
import com.gemwallet.android.cases.banners.HasMultiSign
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.service.store.database.BannersDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.domains.asset.isStakeable
import com.gemwallet.android.domains.banner.BannerAction
import com.gemwallet.android.ext.hasPerpetualsSupport
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.NotificationsAvailable
import com.gemwallet.android.model.getStakedAmount
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemBannerAction
import uniffi.gemstone.GemBannerContext
import uniffi.gemstone.GemBannerItem
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemBannerService
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class BannersRepository(
    private val assetRepository: AssetsRepository,
    private val bannersDao: BannersDao,
    private val userConfig: UserConfig,
    private val notificationsAvailable: NotificationsAvailable,
    private val bannerService: GemBannerService,
) : GetBannersCase, BannerActionCase, HasMultiSign {

    override suspend fun getActiveBanners(wallet: Wallet?, asset: Asset?, isGlobal: Boolean): List<Banner> = withContext(Dispatchers.IO) {
        val assetInfo = asset?.id?.let { assetRepository.getAssetInfo(it).firstOrNull() }
        val sceneWallet = wallet.takeUnless { isGlobal }
        val generated = bannerService.activeEvents(
            walletId = sceneWallet?.id?.id,
            assetId = asset?.id?.toIdentifier(),
            context = bannerContext(sceneWallet, assetInfo),
        ).map { event ->
            Banner(
                walletId = sceneWallet?.id,
                asset = assetInfo?.asset,
                state = BannerState.Active,
                event = event.decodeJson(),
            )
        }
        val stored = when {
            asset != null -> bannersDao.getAssetBanners(
                walletId = wallet?.id?.id,
                assetId = asset.id.toIdentifier(),
            )
            wallet != null -> bannersDao.getWalletBanners(wallet.id.id, listOf(BannerEvent.AccountBlockedMultiSignature))
            else -> emptyList()
        }.map { it.toDTO(asset) }
        val banners = stored + generated
        bannerService.visibleBanners(
            stored = banners.map { GemBannerItem(event = it.event.toJson(), state = it.state.toJson()) },
            context = bannerContext(wallet, assetInfo),
        ).map { item ->
            val event = item.event.decodeJson<BannerEvent>()
            banners.firstOrNull { it.event == event }
                ?: Banner(walletId = sceneWallet?.id, asset = assetInfo?.asset, state = item.state.decodeJson(), event = event)
        }
    }

    override suspend fun applyBannerAction(banner: Banner, action: BannerAction) = withContext(Dispatchers.IO) {
        bannerService.applyAction(banner.toGemKey(), action.toGem())
    }

    override fun hasMultiSign(wallet: Wallet): Flow<Boolean> {
        return bannersDao.getMultisign(wallet.id.id).mapLatest { it.isNotEmpty() }
    }

    private suspend fun bannerContext(wallet: Wallet?, assetInfo: AssetInfo?) = GemBannerContext(
        hasWallet = wallet != null,
        hasAsset = assetInfo != null,
        isStakeable = assetInfo?.asset?.isStakeable == true,
        hasStakeBalance = (assetInfo?.balance?.balance?.getStakedAmount() ?: BigInteger.ZERO) > BigInteger.ZERO,
        hasAvailableBalance = (assetInfo?.balance?.balance?.available?.toBigIntegerOrNull() ?: BigInteger.ZERO) > BigInteger.ZERO,
        isAssetActivated = assetInfo?.balance?.isActive != false,
        assetRankScore = assetInfo?.metadata?.rankScore,
        hasPerpetualsSupport = wallet?.hasPerpetualsSupport == true,
        isWalletEmpty = false,
        notificationsAvailable = notificationsAvailable,
        launchCount = userConfig.getLaunchNumber().toUInt(),
    )
}

private fun Banner.toGemKey() = GemBannerKey(
    walletId = walletId?.id,
    assetId = asset?.id?.toIdentifier(),
    event = event.toJson(),
)

private fun BannerAction.toGem(): GemBannerAction = when (this) {
    is BannerAction.Event -> GemBannerAction.Event(event.toJson())
    BannerAction.Close -> GemBannerAction.Close
}
