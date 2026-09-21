package com.gemwallet.android.features.create_wallet.views

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.create_wallet.components.WordChip
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.CenteredDescriptionText
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.screen.PhraseLayout
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.phraseRows
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.theme.SceneSizing
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WindowDimension
import com.gemwallet.android.ui.theme.isCompactDimension
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.sceneContentPaddingValues
import com.gemwallet.android.ui.theme.space8
import uniffi.gemstone.GemVerifyPhraseViewState

private const val wordsPerGroup = 4

private const val verifyGroupCount = 3

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun CheckPhrase(state: GemVerifyPhraseViewState, loading: Boolean, onPick: (Int) -> Boolean, onDone: () -> Unit, onCancel: () -> Unit) {
    val isSmallScreen = isCompactDimension(WindowDimension.Height)
    val progress = state.nextIndex?.toInt() ?: state.verified.size
    val choices = state.choices.withIndex().toList()

    Scene(
        title = stringResource(id = R.string.transfer_confirm),
        onClose = onCancel,
        padding = sceneContentPaddingValues(horizontalOnly = true),
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.common_continue),
                state = buttonState(enabled = state.isComplete, loading = loading),
            ) {
                onDone()
            }
        },
    ) {
        Column(
            modifier = Modifier.verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            CenteredDescriptionText(stringResource(R.string.secret_phrase_confirm_quick_test_title))
            Spacer16()
            PhraseLayout(
                rows = phraseRows(state.verified),
                modifier = Modifier.widthIn(max = SceneSizing.contentMaxWidth),
                highlightIndex = state.nextIndex?.toInt(),
            )
            AnimatedVisibility(visible = !state.isComplete || !isSmallScreen) {
                FlowRow(
                    modifier = Modifier
                        .padding(vertical = paddingDefault)
                        .widthIn(max = SceneSizing.contentMaxWidth)
                        .fillMaxWidth(),
                    maxItemsInEachRow = wordsPerGroup,
                    horizontalArrangement = Arrangement.spacedBy(space8, Alignment.CenterHorizontally),
                    verticalArrangement = Arrangement.spacedBy(space8),
                ) {
                    val visible = if (isSmallScreen) {
                        val group = progress / wordsPerGroup
                        if (group < verifyGroupCount) choices.drop(group * wordsPerGroup).take(wordsPerGroup) else emptyList()
                    } else {
                        choices
                    }
                    visible.forEach { (index, choice) ->
                        WordChip(choice.word, !choice.isPicked) { onPick(index) }
                    }
                }
            }
        }
    }
}
