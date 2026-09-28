package com.gemwallet.android.features.onboarding.presents.create_wallet

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.AppUrl
import com.gemwallet.android.features.onboarding.viewmodels.create_wallet.CreateWalletViewModel
import com.gemwallet.android.ui.DetectScreenshot
import com.gemwallet.android.ui.DisableScreenShooting
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.animation.navigationSlideTransition
import com.gemwallet.android.ui.components.buttons.CopyButton
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.setCopy
import com.gemwallet.android.ui.components.screen.PhraseLayout
import com.gemwallet.android.ui.components.screen.PhraseRow
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.phraseRows
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.theme.SceneSizing
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPaddingValues
import uniffi.gemstone.GemButtonState
import uniffi.gemstone.GemCopy
import uniffi.gemstone.secretPhraseCopy

private val loadingDialogSize = 100.dp

@Composable
fun CreateWalletScreen(onCancel: () -> Unit, onCreated: () -> Unit) {
    DisableScreenShooting()
    DetectScreenshot(AppUrl.howToSecureSecretPhrase)

    val viewModel: CreateWalletViewModel = hiltViewModel()
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val errorText by viewModel.errorText.collectAsStateWithLifecycle()
    val verificationState by viewModel.verificationState.collectAsStateWithLifecycle()
    val phraseRows by viewModel.phraseRows.collectAsStateWithLifecycle()
    val verifiedRows by viewModel.verifiedRows.collectAsStateWithLifecycle()
    val verificationChoices by viewModel.verificationChoices.collectAsStateWithLifecycle()

    BackHandler(uiState.isShowSafeMessage) {
        viewModel.dismissSafeMessage()
    }

    AnimatedContent(
        targetState = uiState.isShowSafeMessage,
        transitionSpec = {
            navigationSlideTransition(forward = targetState)
        },
        label = "phrase",
    ) { state ->
        when (state) {
            true -> verificationState?.let { verification ->
                VerifyPhraseScene(
                    state = verification,
                    choices = verificationChoices,
                    rows = verifiedRows,
                    onPick = viewModel::onPickWord,
                    onDone = { viewModel.createWallet(onCreated) },
                    onCancel = viewModel::dismissSafeMessage,
                )
            }

            false -> CreateWalletScene(
                rows = phraseRows,
                onCopy = viewModel::phraseCopy,
                dataError = errorText,
                onCreate = viewModel::confirmPhrase,
                onCancel = onCancel,
            )
        }
    }
    if (verificationState?.button == GemButtonState.LOADING) {
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
}

@Composable
private fun CreateWalletScene(rows: List<PhraseRow>, onCopy: () -> GemCopy, dataError: String?, onCreate: () -> Unit, onCancel: () -> Unit) {
    val context = LocalContext.current
    val clipboardManager = LocalContext.current.clipboardManager()
    Scene(
        title = stringResource(id = R.string.wallet_new_title),
        onClose = onCancel,
        padding = sceneContentPaddingValues(),
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.common_continue),
                onClick = onCreate,
            )
        },
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            if (dataError != null) {
                Text(text = dataError)
            } else {
                Text(
                    text = stringResource(id = R.string.secret_phrase_save_phrase_safely),
                    textAlign = TextAlign.Center,
                    color = MaterialTheme.colorScheme.secondary,
                )
                Spacer16()
                PhraseLayout(
                    rows = rows,
                    modifier = Modifier.widthIn(max = SceneSizing.contentMaxWidth),
                )
            }
            Spacer16()
            CopyButton(onClick = { clipboardManager.setCopy(context, onCopy()) })
        }
    }
}

@Composable
@Preview
@Preview(name = "Pixel 2", device = Devices.PIXEL_2)
@Preview(name = "Pixel 3", device = Devices.PIXEL_3)
@Preview(name = "Pixel 4", device = Devices.PIXEL_4)
@Preview(name = "Nexus 5", device = Devices.NEXUS_5)
@Preview(name = "Nexus 7", device = Devices.NEXUS_7)
fun PreviewCreateWalletScene() {
    WalletTheme {
        Column {
            CreateWalletScene(
                rows = phraseRows(
                    listOf(
                        "cinnamon", "two", "three", "cinnamon", "five", "six",
                        "seven", "eight", "cinnamon", "ten", "eleven", "twelve",
                    ),
                ),
                onCopy = { secretPhraseCopy(emptyList()) },
                dataError = null,
                onCreate = {},
                onCancel = {},
            )
        }
    }
}
