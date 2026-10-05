package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetWalletHomeState
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.store.queries.AssetFiatValuesQuery
import com.gemwallet.android.data.services.store.queries.BannersQuery
import com.gemwallet.android.data.services.store.queries.NFTQuery
import com.gemwallet.android.data.services.store.queries.PerpetualWalletBalanceQuery
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemPerpetualCollateral
import uniffi.gemstone.GemWalletHomeServiceInterface
import uniffi.gemstone.GemWalletHomeViewState

@OptIn(ExperimentalCoroutinesApi::class)
class GetWalletHomeStateImpl(
    private val getSession: GetSession,
    private val getActiveAssetsInfo: GetActiveAssetsInfo,
    private val assetFiatValuesQuery: AssetFiatValuesQuery,
    private val perpetualWalletBalanceQuery: PerpetualWalletBalanceQuery,
    private val bannersQuery: BannersQuery,
    private val nftQuery: NFTQuery,
    private val userConfig: UserConfig,
    private val walletHomeService: GemWalletHomeServiceInterface,
    scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : GetWalletHomeState {

    private val walletHomeState = getSession().flatMapLatest { session ->
        val wallet = session?.wallet ?: return@flatMapLatest flowOf(null)

        val perpetualCollateral = combine(
            perpetualWalletBalanceQuery(wallet.id, HypercoreUSDC.id),
            userConfig.isPerpetualEnabled(),
        ) { balance, _ -> balance?.let { GemPerpetualCollateral(balance = it.balance.toGem(), price = it.price) } }

        combine(
            assetFiatValuesQuery(wallet.id),
            perpetualCollateral,
            bannersQuery(wallet.id.id, GemConstants.walletBannerEvents),
            getActiveAssetsInfo.assetsInfo(),
            nftQuery(wallet.id.id),
        ) { balances, perpetualBalance, banners, assets, nfts ->
            walletHomeService.viewState(
                wallet = wallet.toGem(),
                balances = balances.map { it.toGem() },
                perpetual = perpetualBalance,
                banners = banners.map { it.toGem() },
                assetIds = assets.map { it.asset.id.toIdentifier() },
                pinnedAssetIds = assets.filter { it.pinned }.map { it.asset.id.toIdentifier() },
                nfts = nfts.map { it.toGem() },
            )
        }
    }.stateIn(scope, SharingStarted.Eagerly, null)

    override fun walletHomeState(): StateFlow<GemWalletHomeViewState?> = walletHomeState
}
