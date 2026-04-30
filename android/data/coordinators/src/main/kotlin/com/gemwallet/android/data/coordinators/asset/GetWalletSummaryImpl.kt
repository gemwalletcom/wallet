package com.gemwallet.android.data.coordinators.asset

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.assets.coordinators.GetWalletSummary
import com.gemwallet.android.cases.banners.HasMultiSign
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.values.EquivalentValue
import com.gemwallet.android.domains.wallet.aggregates.WalletSummaryAggregate
import com.gemwallet.android.ext.isSwapSupport
import com.gemwallet.android.model.format
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
import java.math.MathContext

@OptIn(ExperimentalCoroutinesApi::class)
class GetWalletSummaryImpl(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
    private val hasMultiSign: HasMultiSign,
    private val userConfig: UserConfig,
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetWalletSummary {
    private val walletSummary = sessionRepository.session().flatMapLatest { session ->
        val wallet = session?.wallet ?: return@flatMapLatest flowOf(null)

        combine(
            assetsRepository.getAssetsInfo(),
            hasMultiSign.hasMultiSign(wallet),
            userConfig.isHideBalances(),
        ) { assets, hasMultiSign, hideBalances ->
            val (totalValue, totalChangedValue) = assets.fold(BigDecimal.ZERO to BigDecimal.ZERO) { (total, changed), asset ->
                val currentValue = asset.balance.fiatTotalAmount.toBigDecimal()
                val currentChangedValue = currentValue * ((asset.price?.price?.priceChangePercentage24h ?: 0.0) / 100).toBigDecimal()

                (total + currentValue) to (changed + currentChangedValue)
            }

            WalletSummaryAggregateImpl(
                wallet = wallet,
                displayState = buildWalletSummaryDisplayState(
                    currency = session.currency,
                    totalValue = totalValue,
                    totalChangedValue = totalChangedValue,
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
    totalValue: BigDecimal,
    totalChangedValue: BigDecimal,
): WalletSummaryDisplayState {
    if (totalValue.compareTo(BigDecimal.ZERO) <= 0) {
        return WalletSummaryDisplayState(
            totalValue = currency.format(BigDecimal.ZERO),
            changedValue = null,
        )
    }
    return WalletSummaryDisplayState(
        totalValue = currency.format(totalValue),
        changedValue = WalletSummaryEquivalentValue(
            currency = currency,
            value = totalChangedValue.toDouble(),
            changePercentage = calculateWalletChangedPercentage(
                totalValue = totalValue,
                changedValue = totalChangedValue,
            ),
        ),
    )
}

internal fun calculateWalletChangedPercentage(
    totalValue: BigDecimal,
    changedValue: BigDecimal,
): Double {
    if (totalValue.compareTo(BigDecimal.ZERO) == 0) {
        return 0.0
    }
    return changedValue.multiply(BigDecimal.valueOf(100.0))
        .divide(totalValue, MathContext.DECIMAL128)
        .toDouble()
}

internal class WalletSummaryEquivalentValue(
    override val currency: Currency,
    override val value: Double?,
    override val changePercentage: Double?,
) : EquivalentValue {
    override val valueFormatted: String
        get() {
            val amount = value?.takeIf(Double::isFinite) ?: return ""
            val formatted = currency.format(amount)
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

    override val walletIcon: Any? = when (wallet.type) {
        WalletType.Multicoin -> null
        WalletType.Single,
        WalletType.PrivateKey,
        WalletType.View -> walletAccount?.chain?.getIconUrl()
    }

    override val walletTotalValue: String = displayState.totalValue

    override val changedValue: EquivalentValue? = displayState.changedValue

    override val isSwapAvailable: Boolean = when (wallet.type) {
        WalletType.Multicoin -> true
        WalletType.Single,
        WalletType.PrivateKey -> walletAccount?.chain?.isSwapSupport() == true
        WalletType.View -> false
    }
}
