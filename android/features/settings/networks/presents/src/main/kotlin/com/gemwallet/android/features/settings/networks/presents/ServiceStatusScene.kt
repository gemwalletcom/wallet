@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.settings.networks.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults.Indicator
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.networks.viewmodels.ServiceStatusViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene

@Composable
fun ServiceStatusScene(
    onCancel: () -> Unit,
    viewModel: ServiceStatusViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    Scene(
        title = stringResource(R.string.transaction_status),
        onClose = onCancel,
    ) {
        val pullToRefreshState = rememberPullToRefreshState()
        PullToRefreshBox(
            isRefreshing = state.isRefreshing,
            onRefresh = viewModel::fetch,
            state = pullToRefreshState,
            indicator = {
                if (pullToRefreshState.distanceFraction > 0f || state.isRefreshing) {
                    Indicator(
                        modifier = Modifier.align(Alignment.TopCenter),
                        isRefreshing = state.isRefreshing,
                        state = pullToRefreshState,
                        containerColor = MaterialTheme.colorScheme.background,
                    )
                }
            },
        ) {
            LazyColumn {
                itemsPositioned(state.rows) { position, item ->
                    ServiceStatusItem(
                        model = item,
                        listPosition = position,
                    )
                }
            }
        }
    }
}
