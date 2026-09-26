package com.gemwallet.android.features.wallet_connector.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.qr_scanner.presents.QRScannerModal
import com.gemwallet.android.features.wallet_connector.viewmodels.ConnectionsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.DocsInfoButton
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.getPlainText
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.listSections
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.QRScanType
import kotlinx.coroutines.launch
import uniffi.gemstone.GemConnection
import uniffi.gemstone.GemEmptyStateKind

@Composable
fun ConnectionsScreen(onConnection: (String) -> Unit, onCancel: () -> Unit, viewModel: ConnectionsViewModel = hiltViewModel()) {
    val clipboardManager = LocalContext.current.clipboardManager()
    var scannerShowed by remember { mutableStateOf(false) }

    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val docsUrl by viewModel.docsUrl.collectAsStateWithLifecycle()

    var pairError by remember { mutableStateOf("") }

    val connectionToastText = stringResource(id = R.string.wallet_connect_connection_title)
    val scope = rememberCoroutineScope()
    val snackbar = remember { SnackbarHostState() }

    Scene(
        title = stringResource(id = R.string.wallet_connect_title),
        snackbar = snackbar,
        actions = {
            docsUrl?.let { DocsInfoButton(it) }
        },
        onClose = onCancel,
    ) {
        LazyColumn {
            item {
                ListItem(
                    model = viewModel.pasteListItem,
                    listPosition = ListPosition.First,
                    modifier = Modifier.clickable {
                        viewModel.addPairing(
                            clipboardManager.getPlainText() ?: return@clickable,
                            { scope.launch { snackbar.showSnackbar(connectionToastText, R.drawable.ic_check_circle) } },
                            { pairError = it },
                        )
                    },
                )
            }
            item {
                ListItem(
                    model = viewModel.scanListItem,
                    listPosition = ListPosition.Last,
                    modifier = Modifier.clickable { scannerShowed = true },
                )
            }
            if (sections.isEmpty()) {
                item {
                    EmptyContentView(type = EmptyContentType(GemEmptyStateKind.WALLET_CONNECT), modifier = Modifier.fillParentMaxHeight(0.7f))
                }
            } else {
                listSections(sections) { position, item ->
                    ListItem(
                        model = item.model,
                        listPosition = position,
                        modifier = Modifier.clickable { onConnection(item.id) },
                    )
                }
            }
        }
    }

    QRScannerModal(
        isVisible = scannerShowed,
        scanType = QRScanType.WalletConnect,
        onDismissRequest = { scannerShowed = false },
        onResult = {
            viewModel.addPairing(it, onSuccess = {}, onError = { error -> pairError = error })
            scannerShowed = false
        },
    )

    if (pairError.isNotEmpty()) {
        AlertDialog(
            onDismissRequest = { pairError = "" },
            confirmButton = {
                Button(onClick = { pairError = "" }) { Text(text = stringResource(id = R.string.common_done)) }
            },
            text = { Text(text = pairError) },
        )
    }
}
