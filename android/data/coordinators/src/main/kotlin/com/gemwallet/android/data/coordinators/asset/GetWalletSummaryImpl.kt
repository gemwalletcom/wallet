package com.gemwallet.android.data.coordinators.asset

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneBannerStore
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.values.EquivalentValue
import com.gemwallet.android.domains.wallet.aggregates.WalletSummaryAggregate
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.PriceChangeFormatter
import com.gemwallet.android.model.text
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemBannerRow
import uniffi.gemstone.GemHeaderActions
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.GemWalletHomeServiceInterface
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.walletBannerEvents
import uniffi.gemstone.walletRow
import java.math.BigDecimal
import uniffi.gemstone.TotalFiatValue as GemTotalFiatValue

@OptIn(ExperimentalCoroutinesApi::class)
class GetWalletSummaryImpl(
    private val getSession: GetSession,
    private val assetStore: GemstoneAssetStore,
    private val getPerpetualBalance: GetPerpetualBalance,
    private val bannerStore: GemstoneBannerStore,
    private val userConfig: UserConfig,
    private val walletHomeService: GemWalletHomeServiceInterface,
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetWalletSummary {

    private val walletSummary = getSession().flatMapLatest { session ->
        val wallet = session?.wallet ?: return@flatMapLatest flowOf(null)

        combine(
            assetStore.observeAssetFiatValues(wallet.id.id),
            getPerpetualBalance.getCollateral(),
            bannerStore.observeWalletBanners(wallet.id.id, walletBannerEvents().map { it.toPrimitives() }),
            userConfig.isHideBalances(),
        ) { balances, perpetualBalance, banners, hideBalances ->
            val state = walletHomeService.viewState(
                wallet = wallet.toGem(),
                balances = balances,
                perpetual = perpetualBalance,
                banners = banners.map { it.toDTO().toGem() },
            )

            WalletSummaryAggregateImpl(
                walletRow = walletRow(wallet.toGem()),
                walletTotalValue = state.total.text(),
                changedValue = state.pnl,
                changeTone = state.pnlTone,
                isBalanceHidden = hideBalances,
                headerActions = state.headerActions,
                showCollections = state.showCollections,
                banners = state.visibleBanners,
            )
        }
    }.stateIn(scope, SharingStarted.Eagerly, null)

    override fun getWalletSummary(): Flow<WalletSummaryAggregate?> = walletSummary
}

@Stable
internal class WalletSummaryAggregateImpl(
    override val walletRow: GemWalletRow,
    override val walletTotalValue: String,
    override val changedValue: GemLocalizedText?,
    override val changeTone: GemValueTone,
    override val isBalanceHidden: Boolean,
    override val headerActions: GemHeaderActions,
    override val showCollections: Boolean,
    override val banners: List<GemBannerRow>,
) : WalletSummaryAggregate
