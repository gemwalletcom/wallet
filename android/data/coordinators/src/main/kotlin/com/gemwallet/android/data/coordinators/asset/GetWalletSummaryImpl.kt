package com.gemwallet.android.data.coordinators.asset

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.stores.GemstoneBannerStore
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.application.session.cases.GetSession
import uniffi.gemstone.GemPercentageStyle
import com.gemwallet.android.domains.banner.BannerRow
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.values.EquivalentValue
import com.gemwallet.android.domains.wallet.aggregates.WalletSummaryAggregate
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.stateIn
import java.math.BigDecimal
import uniffi.gemstone.AssetFiatValue as GemAssetFiatValue
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemHeaderActions
import uniffi.gemstone.GemWalletHomeServiceInterface
import uniffi.gemstone.TotalFiatValue as GemTotalFiatValue
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.walletRow

@OptIn(ExperimentalCoroutinesApi::class)
class GetWalletSummaryImpl(
    private val getSession: GetSession,
    private val getWalletAssets: GetWalletAssets,
    private val getPerpetualBalance: GetPerpetualBalance,
    private val bannerStore: GemstoneBannerStore,
    private val userConfig: UserConfig,
    private val walletHomeService: GemWalletHomeServiceInterface,
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetWalletSummary {

    private val walletSummary = getSession().flatMapLatest { session ->
        val wallet = session?.wallet ?: return@flatMapLatest flowOf(null)

        combine(
            getWalletAssets(),
            getPerpetualBalance.getBalance(),
            bannerStore.observeWalletBanners(wallet.id.id, listOf(BannerEvent.AccountBlockedMultiSignature, BannerEvent.Onboarding)),
            userConfig.isHideBalances(),
        ) { assets, perpetualBalance, banners, hideBalances ->
            val state = walletHomeService.viewState(
                wallet = wallet.toGem(),
                balances = assets.map { asset ->
                    GemAssetFiatValue(
                        amount = asset.balance.totalAmount,
                        price = asset.price?.price?.price ?: 0.0,
                        priceChangePercentage24h = asset.price?.price?.priceChangePercentage24h ?: 0.0,
                    )
                },
                perpetual = perpetualBalance?.toGem(),
                banners = banners.map { it.toDTO().toGem() },
                isWalletEmpty = assets.all { it.balance.totalAmount == 0.0 },
            )

            WalletSummaryAggregateImpl(
                walletRow = walletRow(wallet.toGem()),
                displayState = buildWalletSummaryDisplayState(
                    currency = session.currency,
                    total = state.totalValue,
                    showsPnl = state.showsPnl,
                ),
                isBalanceHidden = hideBalances,
                headerActions = state.headerActions,
                showCollections = state.showCollections,
                banners = state.visibleBanners.map { banner ->
                    BannerRow(banner.toPrimitives(), walletHomeService.bannerContent(banner.event, banner.asset))
                },
            )
        }
    }.stateIn(scope, SharingStarted.Eagerly, null)

    override fun getWalletSummary(): Flow<WalletSummaryAggregate?> {
        return walletSummary
    }
}

internal fun buildWalletSummaryDisplayState(
    currency: Currency,
    total: GemTotalFiatValue,
    showsPnl: Boolean,
): WalletSummaryDisplayState {
    val formatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = currency)
    val totalValue = total.value.toBigDecimal()
    if (!showsPnl) {
        return WalletSummaryDisplayState(
            totalValue = formatter.string(totalValue.coerceAtLeast(BigDecimal.ZERO)),
            changedValue = null,
        )
    }
    return WalletSummaryDisplayState(
        totalValue = formatter.string(totalValue),
        changedValue = WalletSummaryEquivalentValue(
            currency = currency,
            value = total.pnlAmount,
            changePercentage = total.pnlPercentage,
        ),
    )
}

internal class WalletSummaryEquivalentValue(
    override val currency: Currency,
    override val value: Double?,
    override val changePercentage: Double?,
) : EquivalentValue {
    override val valueFormatted: String = value?.takeIf(Double::isFinite)?.let { amount ->
        val formatted = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = currency).string(amount)
        if (amount > 0) "+$formatted" else formatted
    }.orEmpty()

    override val changePercentageFormatted: String = changePercentage.formatAsPercentage(style = GemPercentageStyle.UNSIGNED)
}

internal data class WalletSummaryDisplayState(
    val totalValue: String,
    val changedValue: EquivalentValue?,
)

@Stable
internal class WalletSummaryAggregateImpl(
    override val walletRow: GemWalletRow,
    displayState: WalletSummaryDisplayState,
    override val isBalanceHidden: Boolean,
    override val headerActions: GemHeaderActions,
    override val showCollections: Boolean,
    override val banners: List<BannerRow>,
) : WalletSummaryAggregate {
    override val walletTotalValue: String = displayState.totalValue

    override val changedValue: EquivalentValue? = displayState.changedValue

}
