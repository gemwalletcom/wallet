package com.gemwallet.android.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import com.gemwallet.android.ui.components.image.AvatarScale
import com.gemwallet.android.ui.components.image.EmojiView
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall

private const val EMOJI_COLUMNS = 4

@Composable
fun EmojiPickerGrid(
    emojis: List<String>,
    onSelect: (String) -> Unit,
    background: Color,
    modifier: Modifier = Modifier,
) {
    LazyVerticalGrid(
        columns = GridCells.Fixed(EMOJI_COLUMNS),
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(paddingDefault),
    ) {
        items(emojis) { emoji ->
            Box(
                modifier = Modifier
                    .aspectRatio(1f)
                    .padding(paddingSmall),
            ) {
                EmojiView(
                    emoji = emoji,
                    modifier = Modifier
                        .fillMaxSize()
                        .clickable { onSelect(emoji) },
                    background = background,
                    scale = AvatarScale.EMOJI,
                )
            }
        }
    }
}
