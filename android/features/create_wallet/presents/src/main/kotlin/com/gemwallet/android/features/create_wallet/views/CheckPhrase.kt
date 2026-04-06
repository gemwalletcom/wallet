package com.gemwallet.android.features.create_wallet.views

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.runtime.Composable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.create_wallet.components.WordChip
import com.gemwallet.android.ui.DisableScreenShooting
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.CenteredDescriptionText
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.screen.PhraseLayout
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.theme.SceneSizing
import com.gemwallet.android.ui.theme.space8
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.WindowDimension
import com.gemwallet.android.ui.theme.isCompactDimension
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.sceneContentPaddingValues
import kotlin.math.min

private const val wordsPerGroup = 4
private const val verifyGroupCount = 3

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun CheckPhrase(
    words: List<String>,
    onDone: () -> Unit,
    onCancel: () -> Unit,
) {
    DisableScreenShooting()

    val random = remember {
        val shuffled = mutableListOf<Pair<Int, String>>()
        for (i in 0..words.size / wordsPerGroup) {
            val part = words.mapIndexed { index, word -> Pair(index, word) }.subList(
                fromIndex = i * wordsPerGroup,
                toIndex = min(i * wordsPerGroup + wordsPerGroup, words.size)
            ).shuffled()
            shuffled.addAll(part)
        }
        shuffled.toList()
    }
    val render = remember {
        val state = mutableStateListOf<String>()
        state.addAll(words.map { "" })
        state
    }
    val result = remember {
        mutableStateListOf<String>()
    }
    val isDone by remember {
        derivedStateOf {
            result.joinToString() == words.joinToString()
        }
    }
    val isSmallScreen = isCompactDimension(WindowDimension.Height)

    val onWordClick: (String) -> Boolean = { word ->
        val index = result.size
        if (words[index] == word) {
            result.add(word)
            render[result.size - 1] = word
            true
        } else {
            false
        }
    }

    Scene(
        title = stringResource(id = R.string.transfer_confirm),
        onClose = onCancel,
        padding = sceneContentPaddingValues(horizontalOnly = true),
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.common_continue),
                enabled = isDone,
            ) {
                onDone()
            }
        }
    ) {
        Column(
            modifier = Modifier.verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            CenteredDescriptionText(stringResource(R.string.secret_phrase_confirm_quick_test_title))
            Spacer16()
            PhraseLayout(
                words = render,
                modifier = Modifier.widthIn(max = SceneSizing.contentMaxWidth),
            )
            AnimatedVisibility(visible = !isDone || !isSmallScreen) {
                FlowRow(
                    modifier = Modifier
                        .padding(vertical = paddingDefault)
                        .widthIn(max = SceneSizing.contentMaxWidth)
                        .fillMaxWidth(),
                    maxItemsInEachRow = wordsPerGroup,
                    horizontalArrangement = Arrangement.spacedBy(space8, Alignment.CenterHorizontally),
                    verticalArrangement = Arrangement.spacedBy(space8),
                ) {
                    if (isSmallScreen) {
                        val slice = result.size / wordsPerGroup
                        if (slice < verifyGroupCount) {
                            random
                                .slice(slice * wordsPerGroup..<slice * wordsPerGroup + wordsPerGroup)
                                .forEach { word ->
                                    WordChip(word.second, result.getOrNull(word.first) != word.second, onWordClick)
                                }
                        }
                    } else {
                        random.forEach { word ->
                            WordChip(word.second, result.getOrNull(word.first) != word.second, onWordClick)
                        }
                    }
                }
            }
        }
    }
}
