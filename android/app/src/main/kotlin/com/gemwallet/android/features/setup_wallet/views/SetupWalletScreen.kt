package com.gemwallet.android.features.setup_wallet.views

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.setup_wallet.viewmodels.SetupWalletViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.supportIcon
import com.gemwallet.android.ui.components.list_item.walletItemIconModel
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.theme.extraLargeIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingLarge
import com.wallet.core.primitives.WalletSource

@Composable
fun SetupWalletScreen(
    onComplete: () -> Unit,
) {
    val viewModel: SetupWalletViewModel = hiltViewModel()
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    val handleDone = { onComplete() }

    val title = when (uiState.walletSource) {
        WalletSource.Create -> stringResource(id = R.string.wallet_new_title)
        WalletSource.Import -> stringResource(id = R.string.wallet_import_title)
    }

    Scene(
        title = title,
        backHandle = true,
        actions = {
            IconButton(onClick = handleDone) {
                Icon(imageVector = Icons.Default.Check, contentDescription = "")
            }
        },
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.common_done),
                onClick = handleDone,
            )
        },
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Spacer(modifier = Modifier.size(paddingDefault))
            uiState.walletType?.let { type ->
                IconWithBadge(
                    icon = walletItemIconModel(type = type, walletChain = uiState.walletChain),
                    supportIcon = type.supportIcon(),
                    size = extraLargeIconSize,
                )
            }
            Spacer(modifier = Modifier.size(paddingLarge))
            GemTextField(
                value = uiState.walletName,
                onValueChange = viewModel::onNameChange,
                label = stringResource(id = R.string.wallet_name),
            )
        }
    }
}
