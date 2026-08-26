package com.gemwallet.android.data.repositories.banners

import com.gemwallet.android.cases.banners.CancelBannerCase
import com.gemwallet.android.cases.banners.GetBannersCase
import com.gemwallet.android.cases.banners.HasMultiSign
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.service.store.database.BannersDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.domains.asset.isStakeable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.NotificationsAvailable
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.model.getStakedAmount
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemBannerContext
import uniffi.gemstone.GemBannerService
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class BannersRepository(
    private val assetRepository: AssetsRepository,
    private val bannersDao: BannersDao,
    private val userConfig: UserConfig,
    private val notificationsAvailable: NotificationsAvailable,
    private val bannerService: GemBannerService,
) : GetBannersCase, CancelBannerCase, HasMultiSign {

    override suspend fun getActiveBanners(wallet: Wallet?, asset: Asset?): List<Banner> = withContext(Dispatchers.IO) {
        val assetInfo = asset?.id?.let { assetRepository.getAssetInfo(it).firstOrNull() }
        val generated = bannerService.activeEvents(
            walletId = wallet?.id?.id,
            assetId = asset?.id?.toIdentifier(),
            context = bannerContext(wallet, assetInfo),
        ).map { event ->
            Banner(
                wallet = wallet,
                asset = assetInfo?.asset,
                chain = null,
                state = BannerState.Active,
                event = event.decodeJson(),
            )
        }

        bannersDao.getBanner(
            walletId = wallet?.id?.id ?: "",
            assetId = asset?.id?.toIdentifier() ?: "",
            chain = asset?.id?.chain,
        ).mapNotNull { it.toDTO(wallet, asset) } + generated
    }

    override suspend fun cancelBanner(banner: Banner) = withContext(Dispatchers.IO) {
        bannersDao.saveBanner(banner.toRecord(BannerState.Cancelled))
    }

    private suspend fun bannerContext(wallet: Wallet?, assetInfo: AssetInfo?) = GemBannerContext(
        hasWallet = wallet != null,
        hasAsset = assetInfo != null,
        isStakeable = assetInfo?.asset?.isStakeable == true,
        hasStakeBalance = (assetInfo?.balance?.balance?.getStakedAmount() ?: BigInteger.ZERO) > BigInteger.ZERO,
        isAssetActivated = assetInfo?.balance?.isActive != false,
        notificationsAvailable = notificationsAvailable,
        launchCount = userConfig.getLaunchNumber().toUInt(),
    )

    override fun hasMultiSign(wallet: Wallet): Flow<Boolean> {
        return bannersDao.getMultisign(wallet.id.id).mapLatest { it.isNotEmpty() }
    }
}
