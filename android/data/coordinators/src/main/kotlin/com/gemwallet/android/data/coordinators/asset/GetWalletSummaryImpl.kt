package com.gemwallet.android.data.coordinators.asset

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.banner.cases.HasMultiSign
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.values.EquivalentValue
import com.gemwallet.android.domains.wallet.aggregates.WalletIcon
import com.gemwallet.android.domains.wallet.aggregates.WalletSummaryAggregate
import com.gemwallet.android.ext.isSwapSupport
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.WalletType
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
import uniffi.gemstone.BalanceCalculator
import uniffi.gemstone.TotalFiatValue as GemTotalFiatValue

@OptIn(ExperimentalCoroutinesApi::class)
class GetWalletSummaryImpl(
    private val getSession: GetSession,
    private val getWalletAssets: GetWalletAssets,
    private val getPerpetualBalance: GetPerpetualBalance,
    private val hasMultiSign: HasMultiSign,
    private val userConfig: UserConfig,
    private val balanceCalculator: BalanceCalculator,
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetWalletSummary {

    private val walletSummary = getSession().flatMapLatest { session ->
        val wallet = session?.wallet ?: return@flatMapLatest flowOf(null)

        combine(
            getWalletAssets(),
            getPerpetualBalance.getCollateralIncludedInTotal(),
            hasMultiSign.hasMultiSign(wallet),
            userConfig.isHideBalances(),
        ) { assets, perpetualBalance, hasMultiSign, hideBalances ->
            val balances = assets.map { asset ->
                GemAssetFiatValue(
                    amount = asset.balance.totalAmount,
                    price = asset.price?.price?.price ?: 0.0,
                    priceChangePercentage24h = asset.price?.price?.priceChangePercentage24h ?: 0.0,
                )
            } + listOfNotNull(
                perpetualBalance?.let {
                    GemAssetFiatValue(amount = it.available + it.reserved, price = 1.0, priceChangePercentage24h = 0.0)
                }
            )

            WalletSummaryAggregateImpl(
                wallet = wallet,
                displayState = buildWalletSummaryDisplayState(
                    currency = session.currency,
                    balanceCalculator = balanceCalculator,
                    total = balanceCalculator.totalFiatValue(balances),
                ),
                isBalanceHidden = hideBalances,
                isOperationsAvailable = !hasMultiSign,
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
    balanceCalculator: BalanceCalculator,
): WalletSummaryDisplayState {
    val formatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = currency)
    val totalValue = total.value.toBigDecimal()
    if (!balanceCalculator.showsPnl(total)) {
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
    override val valueFormatted: String
        get() {
            val amount = value?.takeIf(Double::isFinite) ?: return ""
            val formatted = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = currency).string(amount)
            return if (amount > 0) "+$formatted" else formatted
        }

    override val changePercentageFormatted: String
        get() = changePercentage.formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess)
}

internal data class WalletSummaryDisplayState(
    val totalValue: String,
    val changedValue: EquivalentValue?,
)

@Stable
internal class WalletSummaryAggregateImpl(
    wallet: Wallet,
    displayState: WalletSummaryDisplayState,
    override val isBalanceHidden: Boolean,
    override val isOperationsAvailable: Boolean,
) : WalletSummaryAggregate {
    private val walletAccount = wallet.accounts.firstOrNull()

    override val walletType: WalletType = wallet.type

    override val walletName: String = wallet.name

    override val walletIcon: WalletIcon = WalletIcon(
        imageUrl = wallet.imageUrl,
        placeholder = when (wallet.type) {
            WalletType.Multicoin -> null
            WalletType.Single,
            WalletType.PrivateKey,
            WalletType.View -> walletAccount?.chain?.getIconUrl()
        },
    )

    override val walletTotalValue: String = displayState.totalValue

    override val changedValue: EquivalentValue? = displayState.changedValue

    override val isSwapAvailable: Boolean = when (wallet.type) {
        WalletType.Multicoin -> true
        WalletType.Single,
        WalletType.PrivateKey -> walletAccount?.chain?.isSwapSupport() == true
        WalletType.View -> false
    }
}
