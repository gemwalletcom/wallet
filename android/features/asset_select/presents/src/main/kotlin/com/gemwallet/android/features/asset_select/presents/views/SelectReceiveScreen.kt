package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.features.asset_select.viewmodels.ReceiveSelectViewModel
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.iconSize
import com.wallet.core.primitives.AssetId
import com.gemwallet.android.ui.components.clipboard.clipboardManager

@Composable
fun SelectReceiveScreen(
    onCancel: () -> Unit,
    onSelect: ((AssetId) -> Unit)?,
    titleContent: (@Composable () -> Unit)? = null,
    closeIcon: Boolean = false,
    showFilter: Boolean? = null,
    viewModel: ReceiveSelectViewModel = hiltViewModel(),
) {
    val context = LocalContext.current
    val clipboardManager = LocalContext.current.clipboardManager()
    AssetSelectScreen(
        title = stringResource(id = R.string.wallet_receive),
        titleContent = titleContent,
        closeIcon = closeIcon,
        showFilter = showFilter,
        titleBadge = ::getAssetBadge,
        onSelectRecent = onSelect,
        itemTrailing = {
            IconButton(
                onClick = {
                    viewModel.onChangeVisibility(it.asset.id, true)
                    clipboardManager.setPlainText(context, it.accountAddress)
                },
                modifier = Modifier.size(iconSize),
            ) {
                Icon(
                    imageVector = AppIcons.ContentCopyOutlined,
                    contentDescription = "",
                    modifier = Modifier.size(compactIconSize),
                    tint = MaterialTheme.colorScheme.secondary,
                )
            }
        },
        onCancel = onCancel,
        onSelect = onSelect,
        viewModel = viewModel,
    )
}
