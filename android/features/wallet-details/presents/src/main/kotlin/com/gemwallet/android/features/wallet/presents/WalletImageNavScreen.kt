package com.gemwallet.android.features.wallet.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.wallet.viewmodels.WalletImageViewModel

@Composable
fun WalletImageNavScreen(
    onCancel: () -> Unit,
    source: WalletImageSource = WalletImageSource.Wallet,
    viewModel: WalletImageViewModel = hiltViewModel(),
) {
    val wallet by viewModel.wallet.collectAsStateWithLifecycle()
    val nftImages by viewModel.nftImages.collectAsStateWithLifecycle()
    val dismissOnSelect = { if (source == WalletImageSource.Onboarding) onCancel() }

    WalletImageScene(
        wallet = wallet,
        emojis = viewModel.emojis,
        nftImages = nftImages,
        source = source,
        onAction = { action ->
            when (action) {
                is WalletImageAction.SetEmoji -> {
                    viewModel.setEmoji(action.emoji, action.backgroundColor)
                    dismissOnSelect()
                }
                is WalletImageAction.SetNftImage -> viewModel.setNftImage(action.url)
                WalletImageAction.ResetToDefault -> {
                    viewModel.resetToDefault()
                    dismissOnSelect()
                }
                WalletImageAction.Close -> onCancel()
            }
        },
    )
}
