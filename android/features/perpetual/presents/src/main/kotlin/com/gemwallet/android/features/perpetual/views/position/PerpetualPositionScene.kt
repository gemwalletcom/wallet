package com.gemwallet.android.features.perpetual.views.position

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDetailsDataAggregate
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDetailsDataAggregate
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.transaction.transactionsList
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.features.perpetual.views.components.CandleChart
import com.gemwallet.android.features.perpetual.views.components.PerpetualActions
import com.gemwallet.android.features.perpetual.views.components.PerpetualModifyBottomSheet
import com.gemwallet.android.features.perpetual.views.components.PerpetualPositionActions
import com.gemwallet.android.features.perpetual.views.components.perpetualInfo
import com.gemwallet.android.features.perpetual.views.components.positionProperties
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.TransactionId

@Composable
internal fun PerpetualPositionScene(
    perpetual: PerpetualDetailsDataAggregate?,
    position: PerpetualPositionDetailsDataAggregate?,
    transactions: List<TransactionDataAggregate>,
    chartData: List<ChartCandleStick>,
    period: ChartPeriod,
    onAction: (PerpetualDetailsAction) -> Unit,
) {
    var showModifyDialog by remember { mutableStateOf(false) }

    Scene(
        title = perpetual?.name ?: stringResource(R.string.perpetuals_title),
        onClose = { onAction(PerpetualDetailsAction.Close) },
    ) {
        LazyColumn(
            modifier = Modifier.fillMaxSize()
        ) {
            CandleChart(
                data = chartData,
                period = period,
                entry = position?.entryValue,
                liquidation = position?.liquidationValue,
                stopLoss = position?.stopLoss,
                takeProfit = position?.takeProfit,
                onPeriodSelect = { onAction(PerpetualDetailsAction.SelectChartPeriod(it)) },
            )
            positionProperties(position)
            item {
                if (perpetual != null) {
                    if (position == null) {
                        PerpetualActions { onAction(PerpetualDetailsAction.OpenPosition(it)) }
                    } else {
                        PerpetualPositionActions(
                            onModify = { showModifyDialog = true },
                            onClose = { onAction(PerpetualDetailsAction.ClosePosition) },
                        )
                    }
                }
            }
            perpetual?.let { perpetualInfo(it) }
            if (transactions.isNotEmpty()) {
                transactionsList(transactions) { onAction(PerpetualDetailsAction.OpenTransaction(it)) }
            }
        }
    }

    PerpetualModifyBottomSheet(
        isVisible = showModifyDialog,
        onDismiss = { showModifyDialog = false },
        onIncreasePosition = { onAction(PerpetualDetailsAction.IncreasePosition) },
        onReducePosition = { onAction(PerpetualDetailsAction.ReducePosition) },
    )
}

@Preview
@Composable
private fun PerpetualPositionScenePreview() {
    val sampleAsset = Asset(
        id = AssetId(Chain.Bitcoin),
        name = "Bitcoin",
        symbol = "BTC",
        decimals = 8,
        type = AssetType.NATIVE
    )

    val samplePerpetual = object : PerpetualDetailsDataAggregate {
        override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "BTC")
        override val provider: PerpetualProvider = PerpetualProvider.Hypercore
        override val asset: Asset = sampleAsset
        override val name: String = "Bitcoin Perpetual"
        override val dayVolume: String = "$15.00B"
        override val openInterest: String = "$2.50B"
        override val funding: String = "0.01%"
        override val maxLeverage: Int = 40
        override val price: Double = 0.0
        override val identifier: String = "BTC-PERP"
        override val isIsolatedOnly: Boolean = false
    }

    val samplePosition = object : PerpetualPositionDetailsDataAggregate {
        override val positionId: String = "pos-btc-001"
        override val perpetualId: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "BTC")
        override val asset: Asset = sampleAsset
        override val name: String = "BTC"
        override val direction: PerpetualDirection = PerpetualDirection.Long
        override val leverage: Int = 10
        override val marginAmount: String = "$4,771.03"
        override val pnlWithPercentage: String = "+$460.25 (+9.64%)"
        override val pnlState: ValueDirection = ValueDirection.Up
        override val size: String = "$47,250.00"
        override val entryPrice: String = "$94,500.00"
        override val entryValue: Double = 94500.00
        override val liquidationPrice: String = "$85,050.00"
        override val liquidationValue: Double = 85050.00
        override val marginType: PerpetualMarginType = PerpetualMarginType.Cross
        override val fundingPayments: String = "+$12.50"
        override val fundingPaymentsDirection: ValueDirection = ValueDirection.Up
        override val stopLoss: Double = 90050.00
        override val takeProfit: Double = 95000.00
    }

    val now = System.currentTimeMillis()
    val hourInMillis = 60 * 60 * 1000L

    val chartData = List(24) { index ->
        val basePrice = 95000.0
        val variance = (index % 3 - 1) * 500.0
        ChartCandleStick(
            date = now - (23 - index) * hourInMillis,
            open = basePrice + variance,
            high = basePrice + variance + 300.0,
            low = basePrice + variance - 200.0,
            close = basePrice + variance + 100.0,
            volume = 500000000.0 + (index * 10000000.0),
        )
    }

    WalletTheme {
        PerpetualPositionScene(
            perpetual = samplePerpetual,
            position = samplePosition,
            transactions = emptyList(),
            chartData = chartData,
            period = ChartPeriod.Day,
            onAction = {},
        )
    }
}
