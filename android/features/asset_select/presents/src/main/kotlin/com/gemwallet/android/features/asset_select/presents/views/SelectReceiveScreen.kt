package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.features.asset_select.viewmodels.ReceiveSelectViewModel
import com.wallet.core.primitives.AssetId

@Composable
fun SelectReceiveScreen(
    onCancel: () -> Unit,
    onSelect: ((AssetId) -> Unit)?,
    titleContent: (@Composable () -> Unit)? = null,
    closeIcon: Boolean = false,
    showFilter: Boolean? = null,
    viewModel: ReceiveSelectViewModel = hiltViewModel(),
) {
    AssetSelectScreen(
        title = stringResource(id = R.string.wallet_receive),
        titleContent = titleContent,
        closeIcon = closeIcon,
        showFilter = showFilter,
        onSelectRecent = onSelect,
        onCancel = onCancel,
        onSelect = onSelect,
        viewModel = viewModel,
    )
}
