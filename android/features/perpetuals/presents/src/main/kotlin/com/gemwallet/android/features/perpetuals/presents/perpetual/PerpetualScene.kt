package com.gemwallet.android.features.perpetuals.presents.perpetual

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.perpetuals.presents.components.PerpetualActions
import com.gemwallet.android.features.perpetuals.presents.components.PerpetualChartSection
import com.gemwallet.android.features.perpetuals.presents.components.PerpetualModifyBottomSheet
import com.gemwallet.android.features.perpetuals.presents.components.positionProperties
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.list_item.rememberDateSections
import com.gemwallet.android.ui.components.list_item.transaction.transactionsList
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.theme.WalletTheme
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualOrderType
import com.wallet.core.primitives.PerpetualPosition
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.PerpetualTriggerOrder
import uniffi.gemstone.GemCandleChart
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPerpetualButton
import uniffi.gemstone.GemPerpetualButtonRow
import uniffi.gemstone.GemPerpetualDetails
import uniffi.gemstone.GemPerpetualPositionDetail
import uniffi.gemstone.GemPerpetualPositionDetailRow
import uniffi.gemstone.GemPerpetualSection
import uniffi.gemstone.GemTransactionRow
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.candleSession

@Composable
internal fun PerpetualScene(
    details: GemPerpetualDetails?,
    transactions: List<GemTransactionRow>,
    chart: StateViewType<GemCandleChart>,
    period: ChartPeriod,
    isRefreshing: Boolean,
    snackbar: SnackbarHostState? = null,
    onAction: (PerpetualAction) -> Unit,
) {
    var showModifyDialog by remember { mutableStateOf(false) }
    val onButton: (GemPerpetualButton) -> Unit = { action ->
        when (action) {
            GemPerpetualButton.LONG -> onAction(PerpetualAction.OpenPosition(PerpetualDirection.Long))
            GemPerpetualButton.SHORT -> onAction(PerpetualAction.OpenPosition(PerpetualDirection.Short))
            GemPerpetualButton.MODIFY -> showModifyDialog = true
            GemPerpetualButton.CLOSE -> onAction(PerpetualAction.ClosePosition)
            GemPerpetualButton.INCREASE -> onAction(PerpetualAction.IncreasePosition)
            GemPerpetualButton.REDUCE -> onAction(PerpetualAction.ReducePosition)
        }
    }

    Scene(
        title = details?.title ?: stringResource(R.string.perpetuals_title),
        onClose = { onAction(PerpetualAction.Close) },
        snackbar = snackbar,
    ) {
        val transactionSections = rememberDateSections(transactions) { it.createdAt }
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = { onAction(PerpetualAction.Refresh) },
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
            ) {
                item {
                    PerpetualChartSection(
                        state = chart,
                        period = period,
                        onPeriodSelect = { onAction(PerpetualAction.SelectChartPeriod(it)) },
                    )
                }
                details?.sections.orEmpty().forEach { section ->
                    when (section) {
                        is GemPerpetualSection.Position -> {
                            item { SubheaderItem(section.stringRes()) }
                            positionProperties(
                                position = details?.positionRow?.row,
                                rows = section.rows,
                                onAutocloseClick = { onAction(PerpetualAction.Autoclose) },
                            )
                        }

                        is GemPerpetualSection.Info -> {
                            if (section.buttons.isNotEmpty()) {
                                item { PerpetualActions(section.buttons) { onButton(it) } }
                            }
                            item { SubheaderItem(section.stringRes()) }
                            itemsPositioned(section.rows) { rowPosition, row -> GemListRowView(row = row, listPosition = rowPosition) }
                        }
                    }
                }
                if (transactions.isNotEmpty()) {
                    transactionsList(transactionSections) { onAction(PerpetualAction.OpenTransaction(it)) }
                }
            }
        }
    }

    PerpetualModifyBottomSheet(
        isVisible = showModifyDialog,
        title = stringResource(R.string.perpetual_modify),
        buttons = details?.modifyButtons.orEmpty(),
        onDismiss = { showModifyDialog = false },
        onSelect = { onButton(it) },
    )
}

@Preview
@Composable
private fun PerpetualScenePreview() {
    val samplePosition = PerpetualPosition(
        id = "position",
        perpetualId = PerpetualId(PerpetualProvider.Hypercore, "BTC"),
        assetId = AssetId(Chain.Bitcoin),
        size = 0.5,
        sizeValue = 47250.00,
        leverage = 10u,
        entryPrice = 94500.00,
        liquidationPrice = 85050.00,
        marginType = PerpetualMarginType.Cross,
        direction = PerpetualDirection.Long,
        marginAmount = 4771.03,
        takeProfit = PerpetualTriggerOrder(95000.00, PerpetualOrderType.Limit, "tp"),
        stopLoss = PerpetualTriggerOrder(90050.00, PerpetualOrderType.Limit, "sl"),
        pnl = 460.25,
    )

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
        PerpetualScene(
            details = GemPerpetualDetails(
                title = "Bitcoin Perpetual",
                sections = listOf(
                    GemPerpetualSection.Position(
                        rows = listOf(
                            GemPerpetualPositionDetail(GemPerpetualPositionDetailRow.PNL, GemListRow.Text(GemListRowTitle.PNL, "+$460.25 (+9.64%)")),
                            GemPerpetualPositionDetail(
                                GemPerpetualPositionDetailRow.AUTOCLOSE,
                                GemListRow.Lines(
                                    title = GemListRowTitle.AUTO_CLOSE,
                                    lines = listOf(GemLocalizedText.Text("TP $95,000.00"), GemLocalizedText.Text("SL $90,050.00")),
                                    info = GemInfoTopic.AutoClose,
                                ),
                            ),
                        ),
                    ),
                    GemPerpetualSection.Info(
                        buttons = listOf(
                            GemPerpetualButtonRow(GemPerpetualButton.MODIFY, GemValueTone.NEUTRAL),
                            GemPerpetualButtonRow(GemPerpetualButton.CLOSE, GemValueTone.NEGATIVE),
                        ),
                        rows = listOf(GemListRow.Text(GemListRowTitle.DAILY_VOLUME, "$15.00B")),
                    ),
                ),
                modifyButtons = listOf(
                    GemPerpetualButtonRow(GemPerpetualButton.INCREASE, GemValueTone.NEUTRAL),
                    GemPerpetualButtonRow(GemPerpetualButton.REDUCE, GemValueTone.NEGATIVE),
                ),
                position = null,
                positionRow = null,
            ),
            transactions = emptyList(),
            chart = candleSession(ChartPeriod.Day.toGem()).onCandles(chartData.map { it.toGem() }).chart(samplePosition.toGem())?.let { StateViewType.Data(it) } ?: StateViewType.NoData,
            period = ChartPeriod.Day,
            isRefreshing = false,
            onAction = {},
        )
    }
}
