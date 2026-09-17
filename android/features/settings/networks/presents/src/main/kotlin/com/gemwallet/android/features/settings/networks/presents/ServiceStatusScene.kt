@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.settings.networks.presents

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.networks.viewmodels.ServiceStatusViewModel
import com.gemwallet.android.ui.LocalStreamConnected
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun ServiceStatusScene(
    onCancel: () -> Unit,
    viewModel: ServiceStatusViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val isStreamConnected by LocalStreamConnected.current.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) { viewModel.fetch() }

    Scene(
        title = stringResource(R.string.transaction_status),
        onClose = onCancel,
    ) {
        PullToRefreshBox(
            isRefreshing = false,
            onRefresh = viewModel::fetch,
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                itemsPositioned(state.rows) { position, item ->
                    ListItem(model = item.model, listPosition = position)
                }

                item {
                    ListItem(
                        model = ListItemModel(title = "Stream", subtitle = if (isStreamConnected) "🟢" else "🔴"),
                        listPosition = ListPosition.Single,
                    )
                }
            }
        }
    }
}
