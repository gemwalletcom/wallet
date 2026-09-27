package com.gemwallet.android.data.coordinators.asset

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.store.queries.AssetFiatValuesQuery
import com.gemwallet.android.data.services.store.queries.BannersQuery
import com.gemwallet.android.data.services.store.queries.PerpetualWalletBalanceQuery
import com.gemwallet.android.domains.wallet.aggregates.WalletSummary
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemPerpetualCollateral
import uniffi.gemstone.GemWalletHomeServiceInterface
import uniffi.gemstone.GemWalletHomeViewState
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.walletRow

@OptIn(ExperimentalCoroutinesApi::class)
class GetWalletSummaryImpl(
    private val getSession: GetSession,
    private val assetFiatValuesQuery: AssetFiatValuesQuery,
    private val perpetualWalletBalanceQuery: PerpetualWalletBalanceQuery,
    private val bannersQuery: BannersQuery,
    private val userConfig: UserConfig,
    private val walletHomeService: GemWalletHomeServiceInterface,
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetWalletSummary {

    private val walletSummary = getSession().flatMapLatest { session ->
        val wallet = session?.wallet ?: return@flatMapLatest flowOf(null)

        combine(
            assetFiatValuesQuery(wallet.id),
            perpetualWalletBalanceQuery(wallet.id, HypercoreUSDC.id).map { it?.let { GemPerpetualCollateral(balance = it.balance.toGem(), price = it.price) } },
            bannersQuery(wallet.id.id, GemConstants.walletBannerEvents),
            userConfig.isHideBalances(),
            userConfig.isPerpetualEnabled(),
        ) { balances, perpetualBalance, banners, hideBalances, _ ->
            val state = walletHomeService.viewState(
                wallet = wallet.toGem(),
                balances = balances.map { it.toGem() },
                perpetual = perpetualBalance,
                banners = banners.map { it.toGem() },
            )

            WalletSummary(
                state = state,
                walletRow = walletRow(wallet.toGem()),
                isBalanceHidden = hideBalances,
            )
        }
    }.stateIn(scope, SharingStarted.Eagerly, null)

    override fun getWalletSummary(): Flow<WalletSummary?> = walletSummary
}
