package com.gemwallet.android.features.create_wallet.style

import com.gemwallet.android.ui.theme.Emoji
import uniffi.gemstone.GemSecurityReminderItem

internal fun GemSecurityReminderItem.emoji(): String = when (this) {
    GemSecurityReminderItem.KEEP_SAFE -> Emoji.lock
    GemSecurityReminderItem.DO_NOT_SHARE -> Emoji.warning
    GemSecurityReminderItem.NO_RECOVERY -> Emoji.gem
}
