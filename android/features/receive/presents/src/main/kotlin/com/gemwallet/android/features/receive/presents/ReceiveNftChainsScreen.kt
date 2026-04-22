package com.gemwallet.android.features.receive.presents

import androidx.compose.foundation.layout.Row
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.receive.viewmodels.ReceiveNftChainsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.SelectChain
import com.wallet.core.primitives.Chain

@Composable
fun ReceiveNftChainsScreen(
    onCancel: () -> Unit,
    onSelect: (Chain) -> Unit,
    viewModel: ReceiveNftChainsViewModel = hiltViewModel(),
) {
    val context = LocalContext.current
    val clipboardManager = LocalClipboard.current.nativeClipboard
    val chains by viewModel.chains.collectAsStateWithLifecycle()
    SelectChain(
        chains = chains,
        chainFilter = viewModel.chainFilter,
        title = stringResource(id = R.string.wallet_receive_collection),
        trailing = { chain ->
            Row(verticalAlignment = Alignment.CenterVertically) {
                IconButton(
                    onClick = { clipboardManager.setPlainText(context, viewModel.addressFor(chain)) },
                ) {
                    Icon(imageVector = Icons.Default.ContentCopy, contentDescription = null)
                }
                DataBadgeChevron()
            }
        },
        onSelect = onSelect,
        onCancel = onCancel,
    )
}
