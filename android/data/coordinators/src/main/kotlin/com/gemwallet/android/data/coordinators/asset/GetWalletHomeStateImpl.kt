package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetWalletHomeState
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.store.queries.AssetFiatValuesQuery
import com.gemwallet.android.data.services.store.queries.BannersQuery
import com.gemwallet.android.data.services.store.queries.NFTQuery
import com.gemwallet.android.data.services.store.queries.PerpetualWalletBalanceQuery
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
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemNftEntry
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
            bannersQuery(),
            getActiveAssetsInfo.assetsInfo(),
        ) { balances, perpetualBalance, banners, assets ->
            walletHomeService.viewState(
                wallet = wallet.toGem(),
                balances = balances.map { it.toGem() },
                perpetual = perpetualBalance,
                banners = banners.map { it.toGem() },
                assetIds = assets.map { it.asset.id.toIdentifier() },
                pinnedAssetIds = assets.filter { it.pinned }.map { it.asset.id.toIdentifier() },
            )
        }
    }.stateIn(scope, SharingStarted.Eagerly, null)

    private val walletCollections = getSession().map { it?.wallet?.id?.id }.distinctUntilChanged().flatMapLatest { walletId ->
        walletId ?: return@flatMapLatest flowOf(emptyList())
        nftQuery(walletId).map { nfts -> walletHomeService.collections(nfts.map { it.toGem() }) }
    }.stateIn(scope, SharingStarted.Eagerly, emptyList())

    override fun walletHomeState(): StateFlow<GemWalletHomeViewState?> = walletHomeState

    override fun walletCollections(): StateFlow<List<GemNftEntry>> = walletCollections
}
