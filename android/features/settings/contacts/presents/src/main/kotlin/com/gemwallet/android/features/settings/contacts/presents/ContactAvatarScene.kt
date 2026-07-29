package com.gemwallet.android.features.settings.contacts.presents

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.EmojiPickerGrid
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.theme.AvatarEmoji
import com.gemwallet.android.ui.theme.secondaryFaded

@Composable
internal fun ContactAvatarScene(
    onSelect: (String) -> Unit,
    onCancel: () -> Unit,
) {
    Scene(
        title = stringResource(R.string.common_emoji),
        onClose = onCancel,
    ) {
        EmojiPickerGrid(
            emojis = AvatarEmoji.all,
            onSelect = onSelect,
            background = MaterialTheme.colorScheme.secondaryFaded,
        )
    }
}
