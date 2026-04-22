package com.gemwallet.android.features.bridge.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ContentPaste
import androidx.compose.material.icons.filled.QrCodeScanner
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.getShortUrl
import com.gemwallet.android.ext.shortName
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.QrCodeScannerModal
import com.gemwallet.android.ui.components.clipboard.getPlainText
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.gemwallet.android.features.bridge.viewmodels.ConnectionsViewModel
import com.wallet.core.primitives.WalletConnection
import kotlinx.coroutines.launch
import com.gemwallet.android.AppUrl
import uniffi.gemstone.DocsUrl

@Composable
fun ConnectionsScene(
    onConnection: (String) -> Unit,
    onCancel: () -> Unit,
    viewModel: ConnectionsViewModel = hiltViewModel()
) {
    val clipboardManager = LocalClipboard.current.nativeClipboard
    var scannerShowed by remember { mutableStateOf(false) }
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    val connections by viewModel.connections.collectAsStateWithLifecycle()

    var pairError by remember { mutableStateOf("") }

    val connectionToastText = stringResource(id = R.string.wallet_connect_connection_title)
    val scope = rememberCoroutineScope()
    val snackbar = remember { SnackbarHostState() }

    Scene(
        title = stringResource(id = R.string.wallet_connect_title),
        backHandle = true,
        snackbar = snackbar,
        actions = {
            IconButton(onClick = { uriHandler.open(context, AppUrl.docs(DocsUrl.WalletConnect)) }) {
                Icon(imageVector = Icons.Outlined.Info, contentDescription = "WC_INFO")
            }
        },
        onClose = onCancel,
    ) {
        LazyColumn {
            item {
                ListItem(
                    modifier = Modifier.clickable {
                        viewModel.addPairing(
                            clipboardManager.getPlainText() ?: return@clickable,
                            { scope.launch { snackbar.showSnackbar(message = connectionToastText) } },
                            { pairError = it }
                        )
                    },
                    leading = {
                        Icon(
                            imageVector = Icons.Default.ContentPaste,
                            contentDescription = "paste_uri",
                            tint = MaterialTheme.colorScheme.onSurface,
                        )
                    },
                    title = { ListItemTitleText(stringResource(id = R.string.common_paste)) },
                    listPosition = ListPosition.First,
                )
            }
            item {
                ListItem(
                    modifier = Modifier.clickable { scannerShowed = true },
                    leading = {
                        Icon(
                            imageVector = Icons.Default.QrCodeScanner,
                            contentDescription = "scan_qr",
                            tint = MaterialTheme.colorScheme.onSurface,
                        )
                    },
                    title = { ListItemTitleText(stringResource(id = R.string.wallet_scan_qr_code)) },
                    listPosition = ListPosition.Last,
                )
            }
            if (connections.isEmpty()) {
                item {
                    EmptyContentView(type = EmptyContentType.WalletConnect, modifier = Modifier.fillParentMaxHeight(0.7f))
                }
            } else {
                itemsIndexed(connections) { index, item ->
                    ConnectionItem(item, ListPosition.getPosition(index, connections.size), onConnection)
                }
            }
        }
    }

    QrCodeScannerModal(
        isVisible = scannerShowed,
        onDismissRequest = { scannerShowed = false },
        onResult = {
            viewModel.addPairing(it, onSuccess = {}, onError = {})
            scannerShowed = false
        },
    )

    if (pairError.isNotEmpty()) {
        AlertDialog(
            onDismissRequest = {  pairError = "" },
            confirmButton = {
                Button(onClick = { pairError = "" }) { Text(text = stringResource(id = R.string.common_done)) }
            },
            text = { Text(text = pairError) }
        )
    }
}

@Composable
fun ConnectionItem(
    connection: WalletConnection,
    listPosition: ListPosition,
    onClick: ((String) -> Unit)? = null,
) {
    ListItem(
        modifier = if (onClick == null) Modifier else Modifier.clickable { onClick(connection.session.id) },
        leading = {
            val name = connection.session.metadata.shortName
            IconWithBadge(
                connection.session.metadata.icon,
                placeholder = if (name.isEmpty()) "WC" else name[0].toString()
            )
        },
        title = { ListItemTitleText(connection.session.metadata.shortName) },
        subtitle = { ListItemSupportText(connection.session.metadata.url.getShortUrl() ?: connection.session.metadata.url) },
        listPosition = listPosition
    )
}
