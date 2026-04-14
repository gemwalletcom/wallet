package com.gemwallet.android.features.buy.views

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.Scene
import com.wallet.core.primitives.FiatTransactionAssetData

@Composable
fun FiatTransactionsScene(
    transactions: List<FiatTransactionAssetData>,
    isRefreshing: Boolean,
    onClose: () -> Unit,
    onRefresh: () -> Unit,
) {
    val pullToRefreshState = rememberPullToRefreshState()

    Scene(
        title = stringResource(id = R.string.activity_title),
        onClose = onClose,
    ) {
        val uriHandler = LocalUriHandler.current
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = onRefresh,
            state = pullToRefreshState,
            indicator = {
                PullToRefreshDefaults.Indicator(
                    modifier = Modifier.align(Alignment.TopCenter),
                    isRefreshing = isRefreshing,
                    state = pullToRefreshState,
                    containerColor = MaterialTheme.colorScheme.background,
                )
            }
        ) {
            if (transactions.isEmpty()) {
                Column(
                    modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Text(
                        modifier = Modifier.fillMaxWidth().weight(1f)
                            .wrapContentHeight(align = Alignment.CenterVertically),
                        text = stringResource(id = R.string.activity_state_empty_title),
                        textAlign = TextAlign.Center,
                    )
                }
            } else {
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    fiatTransactionsList(
                        items = transactions,
                        onTransactionClick = { info ->
                            info.detailsUrl?.let { url ->
                                try {
                                    uriHandler.openUri(url)
                                } catch (_: Exception) { }
                            }
                        },
                    )
                }
            }
        }
    }
}
