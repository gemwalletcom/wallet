package com.gemwallet.android.features.wallet.presents

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.features.wallet.viewmodels.PrivateKeyAccountsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.Chain

@Composable
fun PrivateKeyAccountsScreen(onSelect: (Chain) -> Unit, onCancel: () -> Unit, viewModel: PrivateKeyAccountsViewModel = hiltViewModel()) {
    val result by viewModel.accounts.collectAsStateWithLifecycle()
    val title = stringResource(R.string.common_private_key)
    val accounts = result?.getOrNull()

    if (accounts == null) {
        if (result == null) {
            LoadingScene(title = title, onCancel = onCancel)
        } else {
            SecretDataErrorScene(title = title, onCancel = onCancel)
        }
        return
    }

    Scene(title = title, onClose = onCancel) {
        LazyColumn(modifier = Modifier.fillMaxSize()) {
            item {
                Row(
                    modifier = Modifier
                        .listItem(ListPosition.Single)
                        .fillMaxWidth()
                        .padding(horizontal = paddingDefault, vertical = paddingMiddle),
                    horizontalArrangement = Arrangement.spacedBy(paddingSmall),
                    verticalAlignment = Alignment.Top,
                ) {
                    Icon(AppIcons.Info, contentDescription = null, tint = MaterialTheme.colorScheme.primary)
                    Text(stringResource(R.string.private_key_accounts_info), color = MaterialTheme.colorScheme.secondary)
                }
            }
            item { SubheaderItem(R.string.private_key_select_account) }
            itemsIndexed(accounts) { index, account ->
                ChainItem(
                    title = account.row.title,
                    icon = ListItemImage.Asset(account.row.icon),
                    subtitle = account.address,
                    listPosition = ListPosition.getPosition(index, accounts.size),
                    trailing = { DataBadgeChevron() },
                    onClick = { onSelect(account.row.chain.requireChain()) },
                )
            }
        }
    }
}
