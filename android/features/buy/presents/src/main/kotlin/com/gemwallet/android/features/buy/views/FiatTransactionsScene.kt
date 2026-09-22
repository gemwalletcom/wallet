package com.gemwallet.android.features.buy.views

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.buy.viewmodels.models.FiatTransactionRowUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.rememberDateSections
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import uniffi.gemstone.GemListRow

@Composable
fun FiatTransactionsScene(transactions: List<FiatTransactionRowUIModel>, errorRow: GemListRow?, isRefreshing: Boolean, onClose: () -> Unit, onRefresh: () -> Unit) {
    Scene(
        title = stringResource(id = R.string.activity_title),
        onClose = onClose,
    ) {
        val uriHandler = LocalUriHandler.current
        val context = LocalContext.current
        val sections = rememberDateSections(transactions) { it.data.createdAt }
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = onRefresh,
        ) {
            if (transactions.isEmpty()) {
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    item {
                        if (errorRow == null) {
                            EmptyContentView(type = EmptyContentType.Activity(), modifier = Modifier.fillParentMaxSize())
                        } else {
                            GemListRowView(row = errorRow, listPosition = ListPosition.Single)
                        }
                    }
                }
            } else {
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    fiatTransactionsList(
                        sections = sections,
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
