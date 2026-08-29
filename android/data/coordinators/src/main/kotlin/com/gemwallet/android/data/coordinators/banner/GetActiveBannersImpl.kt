package com.gemwallet.android.data.coordinators.banner

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.BannersDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.domains.asset.isStakeable
import com.gemwallet.android.ext.hasPerpetualsSupport
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.getStakedAmount
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemBannerContext
import uniffi.gemstone.GemBannerItem
import uniffi.gemstone.GemBannerService
import java.math.BigInteger

class GetActiveBannersImpl(
    private val sessionRepository: SessionRepository,
    private val getAssetInfo: GetAssetInfo,
    private val bannersDao: BannersDao,
    private val bannerService: GemBannerService,
) : GetActiveBanners {

    override suspend fun invoke(asset: Asset?, isGlobal: Boolean): List<Banner> = withContext(Dispatchers.IO) {
        val wallet = sessionRepository.session().firstOrNull()?.wallet
        val assetInfo = asset?.id?.let { getAssetInfo(it).firstOrNull() }
        val sceneWallet = wallet.takeUnless { isGlobal }
        val stored = when {
            asset != null -> bannersDao.getAssetBanners(
                walletId = wallet?.id?.id,
                assetId = asset.id.toIdentifier(),
            )
            wallet != null -> bannersDao.getWalletBanners(wallet.id.id, listOf(BannerEvent.AccountBlockedMultiSignature))
            else -> emptyList()
        }.map { it.toDTO(asset) }
        bannerService.visibleBanners(
            stored = stored.map { GemBannerItem(event = it.event.toJson(), state = it.state.toJson()) },
            context = bannerContext(wallet, assetInfo),
        ).map { item ->
            val event = item.event.decodeJson<BannerEvent>()
            stored.firstOrNull { it.event == event }
                ?: Banner(walletId = sceneWallet?.id, asset = assetInfo?.asset, state = item.state.decodeJson(), event = event)
        }
    }

    private fun bannerContext(wallet: Wallet?, assetInfo: AssetInfo?) = GemBannerContext(
        hasWallet = wallet != null,
        hasAsset = assetInfo != null,
        isStakeable = assetInfo?.asset?.isStakeable == true,
        hasStakeBalance = (assetInfo?.balance?.balance?.getStakedAmount() ?: BigInteger.ZERO) > BigInteger.ZERO,
        hasAvailableBalance = (assetInfo?.balance?.balance?.available?.toBigIntegerOrNull() ?: BigInteger.ZERO) > BigInteger.ZERO,
        isAssetActivated = assetInfo?.balance?.isActive != false,
        assetRankScore = assetInfo?.metadata?.rankScore,
        hasPerpetualsSupport = wallet?.hasPerpetualsSupport == true,
        isWalletEmpty = false,
    )
}
