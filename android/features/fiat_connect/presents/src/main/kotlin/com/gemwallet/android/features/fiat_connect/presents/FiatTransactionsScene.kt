package com.gemwallet.android.features.fiat_connect.presents

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.rememberDateSections
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import uniffi.gemstone.GemFiatTransactionRow
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemListRow

@Composable
fun FiatTransactionsScene(transactions: List<GemFiatTransactionRow>, phase: GemListPhase, isRefreshing: Boolean, onClose: () -> Unit, onRefresh: () -> Unit) {
    Scene(
        title = stringResource(id = R.string.activity_title),
        onClose = onClose,
    ) {
        val uriHandler = LocalUriHandler.current
        val context = LocalContext.current
        val sections = rememberDateSections(transactions) { it.createdAt }
        PullToRefreshBox(
            isRefreshing = isRefreshing,
            onRefresh = onRefresh,
        ) {
            when (phase) {
                is GemListPhase.Empty -> LazyColumn(modifier = Modifier.fillMaxSize()) {
                    item { EmptyContentView(state = phase.state, modifier = Modifier.fillParentMaxSize()) }
                }

                is GemListPhase.Error -> LazyColumn(modifier = Modifier.fillMaxSize()) {
                    item { GemListRowView(row = GemListRow.Error(phase.error), listPosition = ListPosition.Single) }
                }

                GemListPhase.Rows -> LazyColumn(modifier = Modifier.fillMaxSize()) {
                    fiatTransactionsList(
                        sections = sections,
                        onTransactionClick = { row ->
                            row.detailsUrl?.let { url ->
                                uriHandler.open(context, url)
                            }
                        },
                    )
                }
            }
        }
    }
}
