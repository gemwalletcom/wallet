package com.gemwallet.android.features.asset.presents.address

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.asset.viewmodels.address.AddressDetailsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.gemListSections
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.wallet.core.primitives.ChainAddress
import kotlinx.coroutines.launch

@Composable
fun AddressDetailsSheet(chainAddress: ChainAddress?, onDismiss: () -> Unit) {
    ModalBottomSheet(
        item = chainAddress,
        onDismissRequest = onDismiss,
        expansion = SheetExpansion.Full,
        title = { stringResource(R.string.common_address) },
    ) {
        AddressDetailsContent(it)
    }
}

@Composable
private fun AddressDetailsContent(chainAddress: ChainAddress) {
    val viewModel = hiltViewModel<AddressDetailsViewModel, AddressDetailsViewModel.Factory>(
        key = "${chainAddress.chain.string}:${chainAddress.address}",
    ) { it.create(chainAddress) }
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val scope = rememberCoroutineScope()
    var isRefreshing by remember { mutableStateOf(false) }

    LaunchedEffect(chainAddress) {
        viewModel.refresh()
    }

    PullToRefreshBox(
        modifier = Modifier.fillMaxSize(),
        isRefreshing = isRefreshing,
        onRefresh = {
            scope.launch {
                isRefreshing = true
                viewModel.refresh()
                isRefreshing = false
            }
        },
    ) {
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            gemListSections(sections)
        }
    }
}
