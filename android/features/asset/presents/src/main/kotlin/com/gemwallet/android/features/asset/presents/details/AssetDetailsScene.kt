package com.gemwallet.android.features.asset.presents.details

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.getReserveBalanceUrl
import com.gemwallet.android.ui.components.list_item.energyItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.list_item.transaction.transactionsList
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.open
import com.gemwallet.android.features.asset.presents.details.components.AssetDetailsMenu
import com.gemwallet.android.features.asset.presents.details.components.AssetHeadItem
import com.gemwallet.android.features.asset.presents.details.components.BalancePropertyItem
import com.gemwallet.android.features.asset.presents.details.components.BannerItem
import com.gemwallet.android.features.asset.presents.details.components.EmptyTransactionsItem
import com.gemwallet.android.features.asset.presents.details.components.balancesHeader
import com.gemwallet.android.features.asset.presents.details.components.manageAssetItem
import com.gemwallet.android.features.asset.presents.details.components.network
import com.gemwallet.android.features.asset.presents.details.components.price
import com.gemwallet.android.features.asset.presents.details.components.status
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.WalletType

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun AssetDetailsScene(
    uiState: AssetInfoUIModel,
    transactions: List<TransactionDataAggregate>,
    priceAlertEnabled: Boolean,
    priceAlertsCount: Int,
    isRefreshing: Boolean,
    isOperationEnabled: Boolean,
    onAction: (AssetDetailsAction) -> Unit,
) {
    val pullToRefreshState = rememberPullToRefreshState()
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val snackBar = remember { SnackbarHostState() }
    val swapAction: (() -> Unit)? = if (uiState.isSwapEnabled && uiState.accountInfoUIModel.walletType != WalletType.View) {
        { onAction(AssetDetailsAction.Swap(uiState.asset.id, if (uiState.asset.type == AssetType.NATIVE) null else uiState.asset.id.chain.asset().id)) }
    } else {
        null
    }

    Scene(
        titleContent = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = uiState.name,
                    maxLines = 1,
                    overflow = TextOverflow.MiddleEllipsis
                )
            }
        },
        progress = null,
        actions = {
            AssetDetailsMenu(
                uiState = uiState,
                priceAlertEnabled = priceAlertEnabled,
                snackBar = snackBar,
                onPriceAlert = { onAction(AssetDetailsAction.TogglePriceAlert(it)) },
            )
        },
        onClose = { onAction(AssetDetailsAction.Close) },
        snackbar = snackBar,
    ) {
        PullToRefreshBox(
            modifier = Modifier.fillMaxSize(),
            isRefreshing = isRefreshing,
            onRefresh = { onAction(AssetDetailsAction.Refresh) },
            state = pullToRefreshState,
            indicator = {
                PullToRefreshDefaults.Indicator(
                    modifier = Modifier.align(Alignment.TopCenter),
                    isRefreshing = isRefreshing,
                    state = pullToRefreshState,
                    containerColor = MaterialTheme.colorScheme.background
                )
            }
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize()
            ) {
                item {
                    AssetHeadItem(
                        uiState = uiState,
                        isOperationEnabled = isOperationEnabled,
                        onTransfer = { onAction(AssetDetailsAction.Transfer(it)) },
                        onReceive = { onAction(AssetDetailsAction.Receive(it)) },
                        onBuy = { onAction(AssetDetailsAction.Buy(it)) },
                        onSwap = swapAction,
                    )
                }
                item { BannerItem(uiState.assetInfo, { onAction(AssetDetailsAction.Stake(it)) }, { onAction(AssetDetailsAction.Confirm(it)) }) }
                manageAssetItem(uiState.assetInfo, { onAction(AssetDetailsAction.Pin) }, { onAction(AssetDetailsAction.Add) })
                status(uiState.asset, uiState.assetInfo.rank)
                price(uiState, priceAlertsCount, onChart = { onAction(AssetDetailsAction.OpenChart(it)) }, onPriceAlerts = { onAction(AssetDetailsAction.OpenPriceAlerts(it)) })
                network(uiState) { onAction(AssetDetailsAction.OpenNetwork(it)) }
                balancesHeader(uiState.accountInfoUIModel)
                itemsPositioned(uiState.accountInfoUIModel.balances) { position, item ->
                    BalancePropertyItem(
                        title = item.type.label,
                        balance = item.value,
                        listPosition = position,
                        onAction = when (item.type) {
                            AssetInfoUIModel.BalanceViewType.Available -> null
                            AssetInfoUIModel.BalanceViewType.Stake -> {
                                { onAction(AssetDetailsAction.Stake(uiState.asset.id)) }
                            }

                            AssetInfoUIModel.BalanceViewType.Reserved -> {
                                {
                                    uiState.asset.id.chain.getReserveBalanceUrl()
                                        ?.let { uriHandler.open(context, it) }
                                }
                            }
                        }
                    )
                }
                energyItem(uiState.accountInfoUIModel.balanceMetadata)
                item {
                    EmptyTransactionsItem(
                        size = transactions.size,
                        symbol = uiState.asset.symbol,
                        isViewOnly = uiState.accountInfoUIModel.walletType == WalletType.View,
                        onBuy = if (uiState.isBuyEnabled) { { onAction(AssetDetailsAction.Buy(uiState.asset.id)) } } else null,
                        onSwap = if (!uiState.isBuyEnabled) swapAction else null,
                    )
                }
                transactionsList(transactions) { onAction(AssetDetailsAction.OpenTransaction(it)) }
            }
        }
    }
}
