package com.gemwallet.android.features.onboarding.presents.import_wallet

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.PrimaryTabRow
import androidx.compose.material3.SuggestionChip
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.onboarding.presents.import_wallet.components.ImportInput
import com.gemwallet.android.features.onboarding.presents.import_wallet.components.ImportKindTab
import com.gemwallet.android.features.onboarding.viewmodels.import_wallet.ImportInputUIModel
import com.gemwallet.android.features.onboarding.viewmodels.import_wallet.ImportTabUIModel
import com.gemwallet.android.features.onboarding.viewmodels.import_wallet.ImportWalletViewModel
import com.gemwallet.android.features.qr_scanner.presents.QRScannerModal
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.ui.DetectScreenshot
import com.gemwallet.android.ui.DisableScreenShooting
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.sectionHeaderItem
import com.gemwallet.android.ui.components.parseMarkdownToAnnotatedString
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.gemwallet.android.ui.theme.space0
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.QRScanType
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemNameIndicator
import uniffi.gemstone.GemWalletImportKind

private val loadingDialogSize = 100.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ImportWalletScreen(importType: ImportType, onImported: () -> Unit, onCancel: () -> Unit) {
    DisableScreenShooting()

    val context = LocalContext.current
    val viewModel = hiltViewModel<ImportWalletViewModel, ImportWalletViewModel.Factory> { it.create(importType) }
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val errorText by viewModel.errorText.collectAsStateWithLifecycle()
    DetectScreenshot(uiState.importType.kind.secretKind())
    val nameResolveIndicator by viewModel.nameResolveIndicator.collectAsStateWithLifecycle()
    val suggestions by viewModel.suggestions.collectAsStateWithLifecycle()
    val isImported by viewModel.isImported.collectAsStateWithLifecycle()
    var isScanning by rememberSaveable { mutableStateOf(false) }

    LaunchedEffect(isImported) {
        if (isImported) {
            viewModel.onImportedHandled()
            onImported()
        }
    }

    ImportWalletScene(
        inputState = viewModel.inputState,
        tabs = uiState.tabs,
        input = uiState.input,
        title = uiState.title,
        showsTabs = uiState.showsTabs,
        showsScan = uiState.showsScan,
        nameResolveIndicator = nameResolveIndicator,
        buttonState = buttonState(loading = uiState.loading),
        suggestions = suggestions,
    ) { action ->
        when (action) {
            ImportWalletAction.Import -> when (uiState.importType.kind) {
                GemWalletImportKind.ADDRESS -> viewModel.import()
                GemWalletImportKind.PHRASE, GemWalletImportKind.PRIVATE_KEY -> context.requestAuth(AuthRequest.Default) { viewModel.import() }
            }

            ImportWalletAction.Scan -> isScanning = true

            is ImportWalletAction.SelectType -> viewModel.importKind(action.type)

            is ImportWalletAction.SelectSuggestion -> viewModel.selectSuggestion(action.word)

            ImportWalletAction.Cancel -> onCancel()
        }
    }
    if (uiState.loading) {
        Dialog(
            onDismissRequest = {},
            DialogProperties(dismissOnBackPress = false, dismissOnClickOutside = false),
        ) {
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier
                    .size(loadingDialogSize)
                    .background(
                        MaterialTheme.colorScheme.background,
                        shape = RoundedCornerShape(paddingSmall),
                    ),
            ) {
                CircularProgressIndicator()
            }
        }
    }
    QRScannerModal(
        isVisible = isScanning,
        scanType = uiState.importType.kind.scanType(),
        onDismissRequest = { isScanning = false },
        onResult = { code ->
            isScanning = false
            viewModel.inputState.setTextAndPlaceCursorAtEnd(code)
        },
    )
    errorText?.let { message ->
        AlertDialog(
            containerColor = MaterialTheme.colorScheme.background,
            onDismissRequest = viewModel::dismissError,
            title = { Text(stringResource(R.string.errors_validation, "")) },
            text = { Text(message) },
            confirmButton = {
                Button(viewModel::dismissError) { Text(stringResource(R.string.common_done)) }
            },
        )
    }
    uiState.existingWalletName?.let { walletName ->
        InfoBottomSheet(
            item = GemInfoTopic.ExistingWalletImported(walletName).infoSheet {
                viewModel.continueExistingWallet(onImported)
            },
            onClose = viewModel::dismissExistingWallet,
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ImportWalletScene(
    inputState: TextFieldState,
    tabs: List<ImportTabUIModel>,
    input: ImportInputUIModel,
    title: String,
    showsTabs: Boolean,
    showsScan: Boolean,
    nameResolveIndicator: GemNameIndicator?,
    buttonState: ButtonState,
    suggestions: List<String>,
    onAction: (ImportWalletAction) -> Unit,
) {
    Scene(
        title = title,
        onClose = { onAction(ImportWalletAction.Cancel) },
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.wallet_import_action),
                state = buttonState,
                onClick = { onAction(ImportWalletAction.Import) },
            )
        },
    ) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
        ) {
            item {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .listItem(ListPosition.Single)
                        .padding(sceneContentPadding())
                        .padding(bottom = space0),
                    verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
                ) {
                    TypeSelection(tabs, showsTabs) { onAction(ImportWalletAction.SelectType(it)) }
                    DataInput(
                        input = input,
                        inputState = inputState,
                        nameResolveIndicator = nameResolveIndicator,
                        suggestions = suggestions,
                        onSelectSuggestion = { onAction(ImportWalletAction.SelectSuggestion(it)) },
                        onScan = if (showsScan) {
                            { onAction(ImportWalletAction.Scan) }
                        } else {
                            null
                        },
                    )
                }
            }
            if (input.showsViewOnlyWarning) {
                item {
                    Text(
                        modifier = Modifier.sectionHeaderItem().testTag("view_only_warning"),
                        text = parseMarkdownToAnnotatedString(
                            stringResource(R.string.wallet_import_address_warning),
                        ),
                        color = MaterialTheme.colorScheme.secondary,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            }
            item { Spacer(modifier = Modifier.size(it.calculateBottomPadding())) }
        }
    }
}

@Composable
private fun DataInput(input: ImportInputUIModel, inputState: TextFieldState, nameResolveIndicator: GemNameIndicator?, suggestions: List<String>, onSelectSuggestion: (String) -> Unit, onScan: (() -> Unit)?) {
    ImportInput(
        state = inputState,
        input = input,
        indicator = nameResolveIndicator,
        onScan = onScan,
    )

    if (suggestions.isNotEmpty() && input.supportsPhraseSuggestions) {
        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(paddingSmall),
        ) {
            items(suggestions) { word ->
                SuggestionChip(
                    modifier = Modifier.testTag("word_suggestion"),
                    onClick = { onSelectSuggestion(word) },
                    label = { Text(text = word) },
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun TypeSelection(tabs: List<ImportTabUIModel>, showsTabs: Boolean, onTypeChange: (ImportType) -> Unit) {
    if (!showsTabs) {
        return
    }
    PrimaryTabRow(
        modifier = Modifier.fillMaxWidth().clip(RoundedCornerShape(paddingHalfSmall)).testTag("import_kinds"),
        selectedTabIndex = tabs.indexOfFirst { it.isSelected }.coerceAtLeast(0),
        indicator = { Box {} },
        containerColor = Color.Transparent,
        divider = {},
    ) {
        tabs.forEach { tab ->
            ImportKindTab(tab, onTypeChange)
        }
    }
    Spacer16()
}

private fun GemWalletImportKind.scanType(): QRScanType = when (this) {
    GemWalletImportKind.PHRASE -> QRScanType.SecretPhrase
    GemWalletImportKind.PRIVATE_KEY -> QRScanType.PrivateKey
    GemWalletImportKind.ADDRESS -> QRScanType.Address
}

@Composable
@Preview(device = Devices.NEXUS_6)
@Preview(device = Devices.NEXUS_7)
@Preview(showBackground = true, device = Devices.NEXUS_7)
@Preview(showBackground = true, device = Devices.NEXUS_5)
@Preview(showBackground = true, device = "spec:width=411dp,height=891dp")
fun PreviewImportAddress() {
    WalletTheme {
        Box(modifier = Modifier.fillMaxSize()) {
            ImportWalletScene(
                inputState = rememberTextFieldState(),
                tabs = listOf(
                    ImportTabUIModel(ImportType(GemWalletImportKind.PHRASE, Chain.Bitcoin), R.string.common_phrase, isSelected = false),
                    ImportTabUIModel(ImportType(GemWalletImportKind.ADDRESS, Chain.Bitcoin), R.string.common_address, isSelected = true),
                ),
                input = ImportInputUIModel(
                    placeholder = R.string.wallet_import_address_field,
                    protectsInput = false,
                    supportsPhraseSuggestions = false,
                    showsViewOnlyWarning = true,
                ),
                title = "Ethereum",
                showsTabs = true,
                showsScan = true,
                nameResolveIndicator = null,
                buttonState = ButtonState.Enabled,
                suggestions = emptyList(),
                onAction = {},
            )
        }
    }
}
