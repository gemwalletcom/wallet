package com.gemwallet.android.features.add_asset.views

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ui.components.animation.navigationSlideTransition
import com.gemwallet.android.ui.components.QrCodeScannerModal
import com.gemwallet.android.ui.components.screen.SelectChain
import com.gemwallet.android.features.add_asset.viewmodels.AddAssetViewModel
import com.gemwallet.android.features.add_asset.viewmodels.models.AddAssetUIState

@Composable
fun AddAssetScreen(
    onFinish: () -> Unit,
    onCancel: () -> Unit,
    viewModel: AddAssetViewModel = hiltViewModel(),
) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val availableChains by viewModel.availableChains.collectAsStateWithLifecycle()
    val chains by viewModel.chains.collectAsStateWithLifecycle()
    val network by viewModel.selectedChain.collectAsStateWithLifecycle()
    val token by viewModel.token.collectAsStateWithLifecycle()
    val searchState by viewModel.searchState.collectAsStateWithLifecycle()
    val explorerLink by viewModel.explorerLink.collectAsStateWithLifecycle()

    BackHandler(uiState.scene != AddAssetUIState.Scene.Form) {
        viewModel.cancelSelectChain()
        viewModel.cancelScan()
    }

    AnimatedContent(
        targetState = uiState.scene == AddAssetUIState.Scene.SelectChain,
        transitionSpec = {
            navigationSlideTransition(forward = targetState)
        },
        label = "phrase"
    ) { isSelectChain ->
        if (isSelectChain) {
            SelectChain(
                chains = chains,
                chainFilter = viewModel.chainFilter,
                onSelect = viewModel::setChain,
                onCancel = viewModel::cancelSelectChain,
            )
        } else {
            AddAssetScene(
                searchState = searchState,
                addressState = viewModel.addressState,
                network = network.asset(),
                token = token,
                explorerLink = explorerLink,
                isLoading = uiState.isLoading,
                canSelectChain = (availableChains?.size ?: 0) > 1,
                onAction = { action ->
                    when (action) {
                        AddAssetAction.Scan -> viewModel.onQrScan()
                        AddAssetAction.Add -> viewModel.addAsset(onFinish)
                        AddAssetAction.SelectChain -> viewModel.selectChain()
                        AddAssetAction.Cancel -> onCancel()
                    }
                },
            )
        }
    }

    QrCodeScannerModal(
        isVisible = uiState.scene == AddAssetUIState.Scene.QrScanner,
        onDismissRequest = viewModel::cancelScan,
        onResult = viewModel::setQrData,
    )
}
