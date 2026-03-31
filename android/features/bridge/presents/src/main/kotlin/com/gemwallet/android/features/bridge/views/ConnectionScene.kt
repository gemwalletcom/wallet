package com.gemwallet.android.features.bridge.views

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
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.features.bridge.viewmodels.ConnectionViewModel
import java.text.DateFormat
import java.util.Date

@Composable
fun ConnectionScene(
    onCancel: () -> Unit,
    viewModel: ConnectionViewModel = hiltViewModel(),
) {
    val connection by viewModel.connection.collectAsStateWithLifecycle()

    Scene(
        title = stringResource(id = R.string.wallet_connect_title),
        backHandle = true,
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
            connection?.let {
                item { ConnectionItem(it, ListPosition.Single) }
                item { PropertyItem(R.string.common_wallet, it.wallet.name, listPosition = ListPosition.First) }
                item {
                    PropertyItem(
                        title = R.string.transaction_date,
                        data = DateFormat.getDateInstance(DateFormat.MEDIUM).format(Date(it.session.expireAt)),
                        listPosition = ListPosition.Last,
                    )
                }
            }
        }
    }
}
