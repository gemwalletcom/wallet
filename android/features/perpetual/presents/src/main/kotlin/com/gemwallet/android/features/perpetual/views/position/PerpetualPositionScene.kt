package com.gemwallet.android.features.perpetual.views.position

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDetailsDataAggregate
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDetailsDataAggregate
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualButtonAction
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualButtonTone
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualButtonUIModel
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualDetailsSectionUIModel
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualPositionRowUIModel
import com.gemwallet.android.features.perpetual.viewmodels.models.PerpetualChartUIModel
import com.gemwallet.android.features.perpetual.views.components.PerpetualActions
import com.gemwallet.android.features.perpetual.views.components.PerpetualChartSection
import com.gemwallet.android.features.perpetual.views.components.PerpetualModifyBottomSheet
import com.gemwallet.android.features.perpetual.views.components.positionProperties
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.chart.CandlestickTooltipUIModel
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.list_item.rememberDateSections
import com.gemwallet.android.ui.components.list_item.transaction.transactionsList
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.theme.WalletTheme
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Perpetual
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualOrderType
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.PerpetualTriggerOrder
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemValueTone

@Composable
internal fun PerpetualPositionScene(
    perpetual: PerpetualDetailsDataAggregate?,
    position: PerpetualPositionDetailsDataAggregate?,
    positionListItem: ListItemModel?,
    transactions: List<TransactionDataAggregate>,
    chart: StateViewType<PerpetualChartUIModel>,
    period: ChartPeriod,
    tooltip: (ChartCandleStick) -> CandlestickTooltipUIModel,
    isRefreshing: Boolean,
    sections: List<PerpetualDetailsSectionUIModel>,
    modifyButtons: List<PerpetualButtonUIModel>,
    snackbar: SnackbarHostState? = null,
    onAction: (PerpetualDetailsAction) -> Unit,
) {
    var showModifyDialog by remember { mutableStateOf(false) }
    val onButton: (PerpetualButtonAction) -> Unit = { action ->
        when (action) {
            PerpetualButtonAction.OpenLong -> onAction(PerpetualDetailsAction.OpenPosition(PerpetualDirection.Long))
            PerpetualButtonAction.OpenShort -> onAction(PerpetualDetailsAction.OpenPosition(PerpetualDirection.Short))
            PerpetualButtonAction.Modify -> showModifyDialog = true
            PerpetualButtonAction.Close -> onAction(PerpetualDetailsAction.ClosePosition)
            PerpetualButtonAction.Increase -> onAction(PerpetualDetailsAction.IncreasePosition)
            PerpetualButtonAction.Reduce -> onAction(PerpetualDetailsAction.ReducePosition)
        }
    }

    Scene(
        title = perpetual?.name ?: stringResource(R.string.perpetuals_title),
        onClose = { onAction(PerpetualDetailsAction.Close) },
        snackbar = snackbar,
    ) {
        val transactionSections = rememberDateSections(transactions) { it.createdAt }
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = { onAction(PerpetualDetailsAction.Refresh) },
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize()
            ) {
                item {
                    PerpetualChartSection(
                        state = chart,
                        period = period,
                        tooltip = tooltip,
                        onPeriodSelect = { onAction(PerpetualDetailsAction.SelectChartPeriod(it)) },
                    )
                }
                sections.forEach { section ->
                    when (section) {
                        is PerpetualDetailsSectionUIModel.Position -> {
                            item { SubheaderItem(section.title) }
                            positionProperties(
                                position = positionListItem,
                                rows = section.rows,
                                onAutocloseClick = { onAction(PerpetualDetailsAction.Autoclose) },
                            )
                        }
                        is PerpetualDetailsSectionUIModel.Info -> {
                            if (section.buttons.isNotEmpty()) {
                                item { PerpetualActions(section.buttons) { onButton(it) } }
                            }
                            item { SubheaderItem(section.title) }
                            itemsPositioned(section.rows) { rowPosition, row -> GemListRowView(row = row, listPosition = rowPosition) }
                        }
                    }
                }
                if (transactions.isNotEmpty()) {
                    transactionsList(transactionSections) { onAction(PerpetualDetailsAction.OpenTransaction(it)) }
                }
            }
        }
    }

    PerpetualModifyBottomSheet(
        isVisible = showModifyDialog,
        title = stringResource(R.string.perpetual_modify),
        buttons = modifyButtons,
        onDismiss = { showModifyDialog = false },
        onSelect = { onButton(it) },
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
        override val perpetual: Perpetual = Perpetual(
            id = PerpetualId(PerpetualProvider.Hypercore, "BTC"),
            name = "BTC",
            provider = PerpetualProvider.Hypercore,
            assetId = sampleAsset.id,
            identifier = "0",
            price = 0.0,
            pricePercentChange24h = 0.0,
            openInterest = 0.0,
            volume24h = 0.0,
            funding = 0.0,
            maxLeverage = 40u.toUByte(),
            isIsolatedOnly = false,
        )
        override val id: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "BTC")
        override val provider: PerpetualProvider = PerpetualProvider.Hypercore
        override val asset: Asset = sampleAsset
        override val name: String = "Bitcoin Perpetual"
        override val maxLeverage: Int = 40
        override val price: Double = 0.0
        override val identifier: String = "BTC-PERP"
        override val isIsolatedOnly: Boolean = false
    }

    val samplePosition = object : PerpetualPositionDetailsDataAggregate {
        override val perpetualId: PerpetualId = PerpetualId(PerpetualProvider.Hypercore, "BTC")
        override val asset: Asset = sampleAsset
        override val title: String = "BTC"
        override val direction: PerpetualDirection = PerpetualDirection.Long
        override val leverage: String = "10x"
        override val marginAmount: String = "$4,771.03"
        override val pnlWithPercentage: String = "+$460.25 (+9.64%)"
        override val pnlState: GemValueTone = GemValueTone.POSITIVE
        override val size: String = "$47,250.00"
        override val entryPrice: String = "$94,500.00"
        override val liquidationPrice: String = "$85,050.00"
        override val marginType: PerpetualMarginType = PerpetualMarginType.Cross
        override val fundingPayments: String = "+$12.50"
        override val fundingPaymentsDirection: GemValueTone = GemValueTone.POSITIVE
        override val stopLoss: Double = 90050.00
        override val takeProfit: Double = 95000.00
        override val position: PerpetualPosition = PerpetualPosition(
            id = "position",
            perpetualId = perpetualId,
            assetId = sampleAsset.id,
            size = 0.5,
            sizeValue = 47250.00,
            leverage = 10u,
            entryPrice = 94500.00,
            liquidationPrice = 85050.00,
            marginType = marginType,
            direction = direction,
            marginAmount = 4771.03,
            takeProfit = PerpetualTriggerOrder(95000.00, PerpetualOrderType.Limit, "tp"),
            stopLoss = PerpetualTriggerOrder(90050.00, PerpetualOrderType.Limit, "sl"),
            pnl = 460.25,
        )
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
            positionListItem = null,
            transactions = emptyList(),
            chart = StateViewType.Data(PerpetualChartUIModel.from(chartData, samplePosition.position, LocalContext.current)),
            tooltip = { CandlestickTooltipUIModel(emptyList(), emptyList()) },
            period = ChartPeriod.Day,
            isRefreshing = false,
            sections = listOf(
                PerpetualDetailsSectionUIModel.Position(
                    title = "Position",
                    rows = listOf(
                        PerpetualPositionRowUIModel.Item(ListItemModel(title = "PnL", subtitle = "+$460.25 (+9.64%)", subtitleStyle = ListItemTextStyle.Positive)),
                        PerpetualPositionRowUIModel.Autoclose(ListItemModel(title = "Auto close", subtitle = "TP $95,000.00", subtitleExtra = "SL $90,050.00")),
                    ),
                ),
                PerpetualDetailsSectionUIModel.Info(
                    title = "Info",
                    buttons = listOf(
                        PerpetualButtonUIModel("Modify", PerpetualButtonAction.Modify, PerpetualButtonTone.Primary),
                        PerpetualButtonUIModel("Close", PerpetualButtonAction.Close, PerpetualButtonTone.Negative),
                    ),
                    rows = listOf(GemListRow.Text(GemListRowTitle.DAILY_VOLUME, "$15.00B")),
                ),
            ),
            modifyButtons = listOf(
                PerpetualButtonUIModel("Increase", PerpetualButtonAction.Increase, PerpetualButtonTone.Primary),
                PerpetualButtonUIModel("Reduce", PerpetualButtonAction.Reduce, PerpetualButtonTone.Negative),
            ),
            onAction = {},
        )
    }
}
