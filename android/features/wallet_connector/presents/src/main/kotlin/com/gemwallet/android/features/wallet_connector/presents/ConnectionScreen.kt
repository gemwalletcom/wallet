package com.gemwallet.android.features.wallet_connector.presents

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.wallet_connector.viewmodels.ConnectionViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun ConnectionScreen(onCancel: () -> Unit, viewModel: ConnectionViewModel = hiltViewModel()) {
    val connectionListItem by viewModel.connectionListItem.collectAsStateWithLifecycle()
    val rows by viewModel.rows.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    Scene(
        title = stringResource(id = R.string.wallet_connect_title),
        snackbar = snackbar,
        mainAction = {
            TextButton(
                modifier = Modifier.fillMaxWidth(),
                colors = ButtonDefaults.textButtonColors(contentColor = MaterialTheme.colorScheme.error),
                onClick = { viewModel.disconnect(onCancel) },
            ) {
                Text(text = stringResource(id = R.string.wallet_connect_disconnect).uppercase())
            }
        },
        onClose = onCancel,
    ) {
        LazyColumn {
            connectionListItem?.let { item { ListItem(model = it, listPosition = ListPosition.Single) } }
            itemsPositioned(rows) { position, row -> GemListRowView(row = row, listPosition = position) }
        }
    }
}
