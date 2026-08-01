package com.gemwallet.android.features.buy.views

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.open
import com.wallet.core.primitives.FiatTransactionAssetData

@Composable
fun FiatTransactionsScene(
    transactions: List<FiatTransactionAssetData>,
    isRefreshing: Boolean,
    onClose: () -> Unit,
    onRefresh: () -> Unit,
) {
    Scene(
        title = stringResource(id = R.string.activity_title),
        onClose = onClose,
    ) {
        val uriHandler = LocalUriHandler.current
        val context = LocalContext.current
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = onRefresh,
        ) {
            if (transactions.isEmpty()) {
                EmptyContentView(type = EmptyContentType.Activity(), modifier = Modifier.fillMaxSize())
            } else {
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    fiatTransactionsList(
                        items = transactions,
                        onTransactionClick = { info ->
                            info.detailsUrl?.let { url ->
                                uriHandler.open(context, url)
                            }
                        },
                    )
                }
            }
        }
    }
}
