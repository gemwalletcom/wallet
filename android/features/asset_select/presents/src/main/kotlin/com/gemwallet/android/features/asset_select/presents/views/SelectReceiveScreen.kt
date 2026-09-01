package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.features.asset_select.viewmodels.AssetSelectViewModel
import com.gemwallet.android.ui.icons.AppIcons
import com.wallet.core.primitives.AssetId
import com.gemwallet.android.ui.components.clipboard.clipboardManager

@Composable
fun SelectReceiveScreen(
    onCancel: () -> Unit,
    onSelect: ((AssetId) -> Unit)?,
    titleContent: (@Composable () -> Unit)? = null,
    closeIcon: Boolean = false,
    showFilter: Boolean = true,
    viewModel: AssetSelectViewModel = hiltViewModel(),
) {
    val context = LocalContext.current
    val clipboardManager = LocalContext.current.clipboardManager()
    AssetSelectScreen(
        title = stringResource(id = R.string.wallet_receive),
        titleContent = titleContent,
        closeIcon = closeIcon,
        showFilter = showFilter,
        titleBadge = ::getAssetBadge,
        recentType = RecentType.Receive,
        onSelectRecent = onSelect,
        itemTrailing = {
            IconButton(
                onClick = {
                    viewModel.onChangeVisibility(it.asset.id, true)
                    clipboardManager.setPlainText(context, viewModel.getAccount(it.asset.id)?.address ?: "")
                }
            ) {
                Icon(imageVector = AppIcons.ContentCopy, contentDescription = "")
            }
        },
        onCancel = onCancel,
        onSelect = onSelect,
        viewModel = viewModel,
    )
}
