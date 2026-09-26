package com.gemwallet.android.features.assets.presents.add

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.asset
import com.gemwallet.android.features.assets.viewmodels.add.AddAssetViewModel
import com.gemwallet.android.features.assets.viewmodels.add.models.AddAssetUIState
import com.gemwallet.android.features.qr_scanner.presents.QRScannerModal
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.animation.navigationSlideTransition
import com.gemwallet.android.ui.components.screen.SelectChain
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.wallet.core.primitives.QRScanType

@Composable
fun AddAssetScreen(onFinish: () -> Unit, onCancel: () -> Unit, viewModel: AddAssetViewModel = hiltViewModel()) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val showsChainPicker by viewModel.showsChainPicker.collectAsStateWithLifecycle()
    val chains by viewModel.chains.collectAsStateWithLifecycle()
    val network by viewModel.selectedChain.collectAsStateWithLifecycle()
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val isSearching by viewModel.isSearching.collectAsStateWithLifecycle()
    val verificationWarningRow by viewModel.verificationWarningRow.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = uiState.error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    BackHandler(uiState.scene != AddAssetUIState.Scene.Form) {
        viewModel.cancelSelectChain()
        viewModel.cancelScan()
    }

    AnimatedContent(
        targetState = uiState.scene == AddAssetUIState.Scene.SelectChain,
        transitionSpec = {
            navigationSlideTransition(forward = targetState)
        },
        label = "phrase",
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
                isSearching = isSearching,
                addressState = viewModel.addressState,
                network = network?.asset(),
                sections = sections,
                verificationWarningRow = verificationWarningRow,
                buttonState = buttonState,
                canSelectChain = showsChainPicker,
                snackbar = snackbar,
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

    QRScannerModal(
        isVisible = uiState.scene == AddAssetUIState.Scene.QrScanner,
        scanType = QRScanType.TokenContract,
        onDismissRequest = viewModel::cancelScan,
        onResult = viewModel::setQrData,
    )
}
