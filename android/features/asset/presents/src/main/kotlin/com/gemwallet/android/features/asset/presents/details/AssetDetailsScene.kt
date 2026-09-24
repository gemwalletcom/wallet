package com.gemwallet.android.features.asset.presents.details

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.features.asset.presents.details.components.AssetDetailRowItem
import com.gemwallet.android.features.asset.presents.details.components.AssetDetailsMenu
import com.gemwallet.android.features.asset.presents.details.components.AssetHeadItem
import com.gemwallet.android.features.asset.presents.details.components.BannerItem
import com.gemwallet.android.features.asset.presents.details.components.EmptyTransactionsItem
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetDetailsAction
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.list_item.property.verificationStatusItem
import com.gemwallet.android.ui.components.list_item.rememberDateSections
import com.gemwallet.android.ui.components.list_item.transaction.transactionsList
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemListRow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun AssetDetailsScene(
    uiState: AssetInfoUIModel,
    transactions: List<TransactionDataAggregate>,
    transactionsErrorRow: GemListRow?,
    isRefreshing: Boolean,
    snackBar: SnackbarHostState = remember { SnackbarHostState() },
    onAction: (AssetDetailsAction) -> Unit,
) {
    val detailsState = uiState.detailsState
    val swapAction: () -> Unit = {
        uiState.swapPayAssetId?.let { payAssetId ->
            onAction(AssetDetailsAction.Swap(fromAssetId = payAssetId, toAssetId = uiState.swapReceiveAssetId))
        }
    }

    Scene(
        titleContent = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = uiState.name,
                    maxLines = 1,
                    overflow = TextOverflow.MiddleEllipsis,
                )
            }
        },
        progress = null,
        actions = {
            AssetDetailsMenu(
                uiState = uiState,
                priceAlert = uiState.priceAlertMenu,
                onPriceAlert = { onAction(AssetDetailsAction.TogglePriceAlert(it)) },
            )
        },
        onClose = { onAction(AssetDetailsAction.Close) },
        snackbar = snackBar,
    ) {
        val transactionSections = rememberDateSections(transactions) { it.createdAt }
        PullToRefreshBox(
            modifier = Modifier.fillMaxSize(),
            isRefreshing = isRefreshing,
            onRefresh = { onAction(AssetDetailsAction.Refresh) },
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
            ) {
                item {
                    AssetHeadItem(
                        uiState = uiState,
                        onTransfer = { onAction(AssetDetailsAction.Transfer(it)) },
                        onReceive = { onAction(AssetDetailsAction.Receive(it)) },
                        onBuy = { onAction(AssetDetailsAction.Buy(it)) },
                        onSwap = swapAction,
                    )
                }
                if (detailsState.showsBanners) {
                    item {
                        BannerItem(
                            banners = uiState.banners,
                            onStake = { onAction(AssetDetailsAction.Stake(uiState.asset.id)) },
                            onActivate = { onAction(AssetDetailsAction.Confirm(ConfirmTransferInput(it))) },
                            onOpenPerpetuals = { onAction(AssetDetailsAction.OpenPerpetuals) },
                            onClose = { onAction(AssetDetailsAction.CloseBanner(it)) },
                        )
                    }
                }
                uiState.verificationStatus?.let { verificationStatusItem(it) }
                uiState.sections.forEach { section ->
                    section.title?.let { title -> item { SubheaderItem(title) } }
                    itemsPositioned(section.rows) { position, row ->
                        AssetDetailRowItem(uiState = uiState, row = row, listPosition = position, onAction = onAction)
                    }
                }
                item {
                    transactionsErrorRow?.let { GemListRowView(row = it, listPosition = ListPosition.Single) } ?: EmptyTransactionsItem(
                        size = transactions.size,
                        symbol = uiState.asset.symbol,
                        isViewOnly = detailsState.isViewOnly,
                        onBuy = if (uiState.emptyTransactions.showsBuy) {
                            { onAction(AssetDetailsAction.Buy(uiState.asset.id)) }
                        } else {
                            null
                        },
                        onSwap = if (uiState.emptyTransactions.showsSwap) swapAction else null,
                    )
                }
                transactionsList(transactionSections) { onAction(AssetDetailsAction.OpenTransaction(it)) }
            }
        }
    }
}
